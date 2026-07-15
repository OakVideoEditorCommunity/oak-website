/**
 * Derives a human-readable package format from an asset filename extension.
 *
 * Used to tell the four Linux packages apart on the download page. Returns an
 * empty string when the filename does not match a known package type, so
 * callers can simply skip rendering it.
 */
export function packageFormat(filename: string): string {
  const lower = filename.toLowerCase()
  if (lower.endsWith('.appimage')) return 'AppImage'
  if (lower.endsWith('.deb')) return 'deb'
  if (lower.endsWith('.rpm')) return 'rpm'
  if (lower.endsWith('.pkg.tar.zst') || lower.endsWith('.pkg.tar.xz') || lower.endsWith('.pkg.tar.gz')) {
    return 'pkg.tar.zst'
  }
  return ''
}
