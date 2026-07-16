import { describe, it, expect } from 'vitest'
import { docsDocPath } from '../app/composables/useDocsVersions'

describe('docsDocPath', () => {
  const latest = '0.2.0'

  it('returns the unprefixed path for the latest version', () => {
    expect(docsDocPath('0.2.0', latest, 'en', 'intro')).toBe('/docs/en/intro')
  })

  it('returns the unprefixed path for empty or null version', () => {
    expect(docsDocPath(null, latest, 'zh', 'intro')).toBe('/docs/zh/intro')
    expect(docsDocPath(undefined, latest, 'zh', 'intro')).toBe('/docs/zh/intro')
    expect(docsDocPath('', latest, 'zh', 'intro')).toBe('/docs/zh/intro')
  })

  it('returns the version-prefixed path for pinned versions', () => {
    expect(docsDocPath('0.1.0', latest, 'en', 'intro')).toBe('/docs/0.1.0/en/intro')
    expect(docsDocPath('0.1.0', latest, 'zh', 'intro')).toBe('/docs/0.1.0/zh/intro')
  })
})
