import { describe, it, expect } from 'vitest'
import { packageFormat } from '../app/composables/usePackageFormat'

describe('packageFormat', () => {
  it('identifies AppImage, deb, rpm, and pkg.tar.zst by extension', () => {
    expect(packageFormat('Oak_Video_Editor-x86_64.AppImage')).toBe('AppImage')
    expect(packageFormat('oak-video-editor-0.4.0-alpha-amd64.deb')).toBe('deb')
    expect(packageFormat('oak-video-editor-0.4.0-alpha-Linux.rpm')).toBe('rpm')
    expect(packageFormat('oak-video-editor-0.4.0_alpha-1-x86_64.pkg.tar.zst')).toBe('pkg.tar.zst')
  })

  it('returns empty string for unknown or non-package filenames', () => {
    expect(packageFormat('Oak-Video-Editor-Windows-x64.exe')).toBe('')
    expect(packageFormat('Oak-Video-Editor-macOS.dmg')).toBe('')
    expect(packageFormat('README.md')).toBe('')
  })
})
