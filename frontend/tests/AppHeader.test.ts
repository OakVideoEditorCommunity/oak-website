import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import AppHeader from '../app/components/AppHeader.vue'

describe('AppHeader', () => {
  it('links the docs nav entry to the external docs site', async () => {
    const wrapper = await mountSuspended(AppHeader)
    const docsLink = wrapper.find('a[href="https://docs.oakvideoeditor.org"]')
    expect(docsLink.exists()).toBe(true)
    expect(docsLink.attributes('target')).toBe('_blank')
    expect(docsLink.attributes('rel')).toContain('noopener')
  })
})
