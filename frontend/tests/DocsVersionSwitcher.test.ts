import { describe, it, expect, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import { flushPromises } from '@vue/test-utils'
import DocsVersionSwitcher from '../app/components/DocsVersionSwitcher.vue'

describe('DocsVersionSwitcher', () => {
  it('lists all versions and marks the latest', async () => {
    vi.stubGlobal('$fetch', vi.fn((url: string) => {
      if (url.includes('/api/v1/docs/versions')) {
        return Promise.resolve({ versions: ['0.2.0', '0.1.0'], latest: '0.2.0' })
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`))
    }))

    const wrapper = await mountSuspended(DocsVersionSwitcher, {
      props: { version: null, lang: 'en', slug: 'intro' },
    })
    await flushPromises()

    const select = wrapper.find('select')
    expect(select.exists()).toBe(true)

    const options = wrapper.findAll('option')
    expect(options.length).toBe(2)
    expect(options[0].text()).toContain('0.2.0')
    expect(options[0].text()).toContain('Latest')
    expect(options[1].text()).toBe('0.1.0')

    vi.unstubAllGlobals()
  })
})
