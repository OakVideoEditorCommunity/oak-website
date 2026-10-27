/**
 * Platform metadata shared by the download page and release cards.
 *
 * The backend classifies release assets into the eight package families we
 * ship. Keep the display order here so the download page is stable regardless
 * of the order GitHub returns assets in.
 */
export const PLATFORM_ORDER = [
  'windows',
  'macos',
  'debian',
  'fedora',
  'arch',
  'openkylin-arm',
  'openkylin-x86',
  'appimage',
] as const

const PLATFORM_LABEL_KEYS: Record<string, string> = {
  windows: 'platform.windows',
  macos: 'platform.macos',
  debian: 'platform.debian',
  fedora: 'platform.fedora',
  arch: 'platform.arch',
  'openkylin-arm': 'platform.openkylinArm',
  'openkylin-x86': 'platform.openkylinX86',
  appimage: 'platform.appimage',
  linux: 'platform.linux',
  unknown: 'platform.unknown',
}

/** A small integer used to sort assets in download-page order. */
export function platformRank(platform: string): number {
  const index = (PLATFORM_ORDER as readonly string[]).indexOf(platform)
  return index === -1 ? PLATFORM_ORDER.length : index
}

/** The i18n key for a platform, or null when it is unknown to the UI. */
export function platformLabelKey(platform: string): string | null {
  return PLATFORM_LABEL_KEYS[platform] ?? null
}

export function usePlatform() {
  const { t } = useI18n()

  function label(platform: string): string {
    const key = platformLabelKey(platform)
    return key ? t(key) : platform
  }

  return { label, rank: platformRank }
}
