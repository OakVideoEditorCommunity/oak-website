import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import HeroSection from '../app/components/HeroSection.vue'
import type { Release } from '../types'

describe('HeroSection', () => {
  it('renders hero title and download link', async () => {
    const latest: Release = {
      id: '550e8400-e29b-41d4-a716-446655440000',
      version: 'v1.0.0',
      tag_name: 'v1.0.0',
      is_prerelease: false,
      assets: [],
    }
    const wrapper = await mountSuspended(HeroSection, {
      props: { latest },
    })
    expect(wrapper.text()).toContain('Oak Video Editor')
  })
})
