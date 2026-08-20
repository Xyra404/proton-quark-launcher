/**
 * Formats total playtime in seconds into a human-readable string for display in the UI.
 * - 0 or undefined -> "Never played"
 * - < 60 seconds -> "< 1m"
 * - >= 60 seconds -> "3h 42m" or "15m"
 */
export function formatPlaytime(seconds?: number): string {
  if (!seconds || seconds <= 0) return 'Never played';
  if (seconds < 60) return '< 1m';

  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}

/**
 * Formats an ISO 8601 UTC date string into a localized medium date/short time format.
 */
export function formatDate(iso?: string): string {
  if (!iso) return 'Never played';
  try {
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short',
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}
