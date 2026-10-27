import { describe, it, expect } from 'vitest'
import {
  PLATFORM_ORDER,
  platformLabelKey,
  platformRank,
} from '../app/composables/usePlatform'

describe('usePlatform', () => {
  it('orders all eight supported package families', () => {
    expect(PLATFORM_ORDER).toEqual([
      'windows',
      'macos',
      'debian',
      'fedora',
      'arch',
      'openkylin-arm',
      'openkylin-x86',
      'appimage',
    ])
  })

  it('maps every supported platform to a label key', () => {
    for (const platform of PLATFORM_ORDER) {
      expect(platformLabelKey(platform)).toBeTruthy()
    }
    expect(platformLabelKey('openkylin-arm')).toBe('platform.openkylinArm')
    expect(platformLabelKey('openkylin-x86')).toBe('platform.openkylinX86')
  })

  it('sorts known platforms before unknown ones', () => {
    expect(platformRank('windows')).toBe(0)
    expect(platformRank('appimage')).toBe(7)
    expect(platformRank('mystery')).toBe(8)
  })
})
