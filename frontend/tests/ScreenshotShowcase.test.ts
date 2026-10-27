import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import ScreenshotShowcase from '../app/components/ScreenshotShowcase.vue'

describe('ScreenshotShowcase', () => {
  it('renders the Oak editor screenshot with descriptive alt text', async () => {
    const wrapper = await mountSuspended(ScreenshotShowcase)
    const image = wrapper.find('img')

    expect(image.exists()).toBe(true)
    expect(image.attributes('src')).toBe('/images/oak-editor-screenshot.webp')
    expect(image.attributes('alt')).toContain('Oak Video Editor')
    expect(image.attributes('width')).toBe('2680')
    expect(image.attributes('height')).toBe('1564')
  })
})
