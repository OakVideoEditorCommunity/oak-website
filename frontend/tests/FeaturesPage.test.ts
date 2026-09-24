import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import FeaturesPage from '../app/pages/features.vue'

describe('features page', () => {
  it('renders all five feature sections', async () => {
    const wrapper = await mountSuspended(FeaturesPage)
    const text = wrapper.text()
    for (const token of ['OpenColorIO', 'OpenFX', 'OpenTimelineIO', 'PQ']) {
      expect(text).toContain(token)
    }
    expect(text).toMatch(/10-?bit/)
    for (const id of ['ocio', 'openfx', 'otio', 'colorMgmt', 'tenBit']) {
      expect(wrapper.find(`#${id}`).exists()).toBe(true)
    }
  })
})
