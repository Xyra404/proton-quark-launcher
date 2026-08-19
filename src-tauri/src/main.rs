// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = proton_quark_launcher_lib::run() {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}
