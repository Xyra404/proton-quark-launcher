use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A GE-Proton release available for download.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtonRelease {
    pub tag: String,
    pub download_url: String,
    pub size_bytes: Option<u64>,
    pub published_at: String,
}

/// Payload emitted to the frontend during a download.
#[derive(Serialize, Clone)]
struct DownloadProgressPayload {
    tag: String,
    bytes_downloaded: u64,
    total_bytes: Option<u64>,
}

/// Payload emitted when a download completes (successfully or not).
#[derive(Serialize, Clone)]
struct DownloadCompletePayload {
    tag: String,
    success: bool,
    error: Option<String>,
}

// ─── GitHub API helpers ───────────────────────────────────────────────────────

/// Raw GitHub release structure — we only need a subset of the fields.
#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("proton-quark-launcher/0.1 (github.com/your-handle/proton-quark-launcher)")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

// ─── Architecture filtering ────────────────────────────────────────────────────
//
// GE-Proton publishes separate releases/assets for different CPU
// architectures (e.g. tag "GE-Proton11-5" for x86_64 vs a sibling release
// or asset tagged/named with "aarch64"/"arm64" for ARM64 Linux desktops).
// Without filtering by architecture, the app can download a build that is
// physically incompatible with the host CPU, which fails at launch time
// with a kernel-level "Exec format error" — not a permissions or extraction
// bug, but a wrong-binary-for-this-CPU problem. This must be caught here,
// before anything is downloaded or listed as installable.

/// Known architecture markers that can appear in a tag name or asset filename.
const FOREIGN_ARCH_MARKERS: &[&str] = &["aarch64", "arm64", "armv7", "armhf", "riscv64"];

/// Returns the current host's architecture as it's likely to appear in
/// release/asset naming (Rust's own `ARCH` constant already matches GitHub's
/// convention closely enough for x86_64/aarch64).
fn host_arch() -> &'static str {
    std::env::consts::ARCH // "x86_64" on virtually all desktop Linux machines
}

/// True if `name` (a tag name or asset filename) does not explicitly target a
/// different architecture than the host. Releases/assets with no
/// architecture marker at all are treated as compatible (older GE-Proton
/// releases only ever shipped one build with no arch suffix).
fn is_compatible_with_host(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    let host = host_arch();

    for marker in FOREIGN_ARCH_MARKERS {
        if name_lower.contains(marker) {
            // The name mentions SOME architecture — only accept it if that
            // architecture is actually the host's.
            return name_lower.contains(host);
        }
    }

    // No architecture marker present at all — assume it's the default
    // (historically x86_64-only) build and accept it.
    true
}

/// Picks the best-matching `.tar.gz` asset for the host architecture out of
/// a release's asset list. Prefers an asset whose name explicitly matches
/// the host arch, falls back to an asset with no arch marker at all, and
/// otherwise returns None (no compatible asset in this release).
fn select_matching_asset(assets: &[GhAsset]) -> Option<&GhAsset> {
    let host = host_arch();

    // First pass: explicit match for the host architecture.
    if let Some(asset) = assets.iter().find(|a| {
        a.name.ends_with(".tar.gz") && a.name.to_lowercase().contains(host)
    }) {
        return Some(asset);
    }

    // Second pass: no architecture marker at all (older, single-arch releases).
    assets.iter().find(|a| {
        a.name.ends_with(".tar.gz") && is_compatible_with_host(&a.name)
    })
}

// ─── Install directory helpers ────────────────────────────────────────────────

/// Returns the ordered list of candidate `compatibilitytools.d` parent directories.
fn compat_tool_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Native Steam installs
        candidates.push(home.join(".steam/steam/compatibilitytools.d"));
        candidates.push(home.join(".local/share/Steam/compatibilitytools.d"));
        // Flatpak Steam
        candidates.push(
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/compatibilitytools.d"),
        );
    }

    candidates
}

/// Resolves the best `compatibilitytools.d` directory to use for a new install.
/// Prefers an already-existing directory; falls back to the second candidate and
/// creates it if necessary.
fn resolve_install_parent(tag: &str) -> Result<PathBuf, String> {
    let dirs = compat_tool_dirs();

    // Prefer first existing compat dir that already exists
    for dir in &dirs {
        if dir.exists() {
            // Also check the final destination doesn't already exist
            let dest = dir.join(tag);
            if dest.exists() {
                return Err(format!(
                    "Proton version '{tag}' is already installed at '{}'.",
                    dest.display()
                ));
            }
            return Ok(dir.clone());
        }
    }

    // None exist yet — create the primary fallback
    let fallback = dirs
        .into_iter()
        .nth(1)
        .ok_or_else(|| "Could not determine home directory.".to_string())?;

    fs::create_dir_all(&fallback).map_err(|e| {
        format!(
            "Failed to create compatibilitytools.d at '{}': {e}",
            fallback.display()
        )
    })?;

    Ok(fallback)
}

/// Safety check: path must be inside one of the known compat tool directories.
fn assert_in_compat_dir(path: &Path) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize '{}': {e}", path.display()))?;

    for dir in compat_tool_dirs() {
        // Try to canonicalize the compat dir itself (may not exist — skip if so)
        if let Ok(canon_dir) = dir.canonicalize() {
            if canonical.starts_with(&canon_dir) {
                return Ok(());
            }
        }
    }

    Err(format!(
        "Safety check failed: '{}' is not inside a known compatibilitytools.d directory. \
         Deletion refused.",
        path.display()
    ))
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Fetches the most recent GE-Proton releases from the GitHub Releases API,
/// filtered to only those compatible with the host's CPU architecture.
#[tauri::command]
pub async fn list_available_proton_downloads() -> Result<Vec<ProtonRelease>, String> {
    let client = build_client()?;

    let response = client
        .get("https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases")
        .query(&[("per_page", "15")])
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Network error fetching releases: {e}"))?;

    if response.status() == 403 || response.status() == 429 {
        return Err(
            "GitHub API rate limit reached. Please wait a few minutes and try again. \
             (No authentication is required — the limit resets automatically.)"
                .to_string(),
        );
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned HTTP {}: {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("unknown")
        ));
    }

    let gh_releases: Vec<GhRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub releases JSON: {e}"))?;

    let releases = gh_releases
        .into_iter()
        // Skip whole releases whose TAG itself targets a different
        // architecture (e.g. "GE-Proton11-5-aarch64" on an x86_64 host).
        .filter(|r| is_compatible_with_host(&r.tag_name))
        .filter_map(|r| {
            // Among this release's assets, pick the one matching the host
            // architecture (handles releases that bundle multiple arch
            // builds as separate assets under one tag).
            let asset = select_matching_asset(&r.assets)?;

            Some(ProtonRelease {
                tag: r.tag_name,
                download_url: asset.browser_download_url.clone(),
                size_bytes: Some(asset.size),
                published_at: r.published_at,
            })
        })
        .collect();

    Ok(releases)
}

/// Downloads and extracts a GE-Proton release, streaming progress events to the frontend.
#[tauri::command]
pub async fn download_proton_version(
    app: AppHandle,
    release: ProtonRelease,
) -> Result<(), String> {
    let tag = release.tag.clone();

    // Defense in depth: even though list_available_proton_downloads() should
    // never surface a mismatched release, re-verify here in case this command
    // is ever called with a stale or manually-constructed ProtonRelease.
    if !is_compatible_with_host(&tag) || !is_compatible_with_host(&release.download_url) {
        return Err(format!(
            "'{tag}' targets a different CPU architecture than this machine ({}). \
             Refusing to download an incompatible build.",
            host_arch()
        ));
    }

    let install_parent = resolve_install_parent(&tag)?;
    let install_dir = install_parent.join(&tag);

    // ── Create a temp file for the download ──────────────────────────────────
    let tmp_path = install_parent.join(format!("{tag}.tar.gz.tmp"));

    let result = do_download_and_extract(&app, &release, &tmp_path, &install_dir).await;

    match result {
        Ok(()) => {
            // Clean up temp file on success
            let _ = fs::remove_file(&tmp_path);

            let _ = app.emit(
                "proton-download-complete",
                DownloadCompletePayload {
                    tag: tag.clone(),
                    success: true,
                    error: None,
                },
            );

            Ok(())
        }
        Err(e) => {
            let _ = app.emit(
                "proton-download-complete",
                DownloadCompletePayload {
                    tag: tag.clone(),
                    success: false,
                    error: Some(e.clone()),
                },
            );

            Err(format!(
                "{e} A partial download may remain at '{}'. \
                 You can delete it manually.",
                tmp_path.display()
            ))
        }
    }
}

async fn do_download_and_extract(
    app: &AppHandle,
    release: &ProtonRelease,
    tmp_path: &Path,
    install_dir: &Path,
) -> Result<(), String> {
    let tag = &release.tag;
    let client = build_client()?;

    // ── Stream download ──────────────────────────────────────────────────────
    let response = client
        .get(&release.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download for '{tag}': {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download server returned HTTP {} for '{tag}'.",
            response.status()
        ));
    }

    let total_bytes = response.content_length();

    let file =
        fs::File::create(tmp_path).map_err(|e| format!("Cannot create temp file: {e}"))?;
    let mut writer = BufWriter::new(file);

    let mut stream = response.bytes_stream();
    let mut bytes_downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download interrupted for '{tag}': {e}"))?;
        writer
            .write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk to temp file: {e}"))?;

        bytes_downloaded += chunk.len() as u64;

        let _ = app.emit(
            "proton-download-progress",
            DownloadProgressPayload {
                tag: tag.clone(),
                bytes_downloaded,
                total_bytes,
            },
        );
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush temp file: {e}"))?;
    drop(writer);

    // ── Extract ──────────────────────────────────────────────────────────────
    let tar_file =
        fs::File::open(tmp_path).map_err(|e| format!("Cannot open downloaded archive: {e}"))?;

    let gz = GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(gz);

    // GE-Proton tarballs always wrap content in a single top-level folder
    // (e.g. GE-Proton9-27/). We strip that component and install directly
    // into `install_dir` instead, so the result is e.g.:
    //   ~/.steam/steam/compatibilitytools.d/GE-Proton9-27/proton  ✓
    fs::create_dir_all(install_dir)
        .map_err(|e| format!("Failed to create install directory: {e}"))?;

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read archive entries: {e}"))?;

    // Re-open to iterate (tar::Archive consumes the iterator on first pass)
    let tar_file2 =
        fs::File::open(tmp_path).map_err(|e| format!("Cannot re-open archive: {e}"))?;
    let gz2 = GzDecoder::new(tar_file2);
    let mut archive2 = tar::Archive::new(gz2);

    // Detect top-level folder name by looking at the first entry
    let mut top_level_dir: Option<String> = None;
    for entry in entries.take(1) {
        if let Ok(e) = entry {
            if let Ok(p) = e.path() {
                let components: Vec<_> = p.components().collect();
                if let Some(Component::Normal(first)) = components.first() {
                    top_level_dir = Some(first.to_string_lossy().into_owned());
                }
            }
        }
    }

    archive2
        .entries()
        .map_err(|e| format!("Failed to read archive (pass 2): {e}"))?
        .filter_map(|e| e.ok())
        .try_for_each(|mut entry| -> Result<(), String> {
            let entry_path = entry
                .path()
                .map_err(|e| format!("Bad archive entry path: {e}"))?
                .into_owned();

            // Strip the top-level component if detected
            let rel_path = if let Some(ref prefix) = top_level_dir {
                let prefix_path = Path::new(prefix);
                entry_path
                    .strip_prefix(prefix_path)
                    .unwrap_or(&entry_path)
                    .to_path_buf()
            } else {
                entry_path
            };

            // Skip the stripped root itself (empty path)
            if rel_path.as_os_str().is_empty() {
                return Ok(());
            }

            let dest = install_dir.join(&rel_path);

            // Safety: dest must remain inside install_dir
            if !dest.starts_with(install_dir) {
                return Err(format!(
                    "Archive contains path traversal entry: '{}'",
                    rel_path.display()
                ));
            }

            // entry.unpack() preserves the archive's original Unix
            // permission bits (including the executable bit on `proton`
            // and other binaries) — do not replace this with manual
            // File::create()+write_all(), which would silently drop them
            // and cause "Permission denied" at launch time instead.
            entry
                .unpack(&dest)
                .map_err(|e| format!("Failed to unpack '{}': {e}", rel_path.display()))?;

            Ok(())
        })?;

    Ok(())
}

/// Deletes an installed Proton version folder.
/// Safety: only allows deletion of paths inside a known compatibilitytools.d directory.
#[tauri::command]
pub fn delete_proton_version(path: String) -> Result<(), String> {
    let p = Path::new(&path);

    if !p.exists() {
        return Err(format!("Path does not exist: '{path}'"));
    }

    if !p.is_dir() {
        return Err(format!("'{path}' is not a directory."));
    }

    assert_in_compat_dir(p)?;

    fs::remove_dir_all(p)
        .map_err(|e| format!("Failed to delete '{}': {e}", p.display()))
}
