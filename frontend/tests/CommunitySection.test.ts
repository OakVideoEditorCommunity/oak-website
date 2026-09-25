import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import CommunitySection from '../app/components/CommunitySection.vue'

describe('CommunitySection', () => {
  it('renders all four community channels', async () => {
    const wrapper = await mountSuspended(CommunitySection)

    const hrefs = wrapper.findAll('a').map(a => a.attributes('href'))
    expect(hrefs).toContain('mailto:dev@oakvideoeditor.org')
    expect(hrefs).toContain('https://www.facebook.com/groups/oakvideoeditor/')
    expect(hrefs).toContain('https://t.me/oakvideoeditor')
    expect(hrefs).toContain('https://github.com/OakVideoEditorCommunity/oak')
  })
})
