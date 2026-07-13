import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import ReleaseCard from '../app/components/ReleaseCard.vue'
import type { Release } from '../types'

describe('ReleaseCard', () => {
  it('renders release version and ready assets', async () => {
    const release: Release = {
      id: '550e8400-e29b-41d4-a716-446655440000',
      version: 'v1.0.0',
      tag_name: 'v1.0.0',
      is_prerelease: false,
      assets: [
        {
          id: '660e8400-e29b-41d4-a716-446655440000',
          platform: 'linux',
          arch: 'x86_64',
          filename: 'test.AppImage',
          size_bytes: 1024 * 1024 * 100,
          sync_status: 'ready',
        },
      ],
    }
    const wrapper = await mountSuspended(ReleaseCard, {
      props: { release },
    })
    expect(wrapper.text()).toContain('v1.0.0')
    expect(wrapper.text()).toContain('Linux')
  })
})
