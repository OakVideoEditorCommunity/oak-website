import { describe, it, expect, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import DownloadButton from '../app/components/DownloadButton.vue'

describe('DownloadButton', () => {
  it('renders a download button', async () => {
    const wrapper = await mountSuspended(DownloadButton, {
      props: {
        releaseId: '550e8400-e29b-41d4-a716-446655440000',
        platform: 'linux',
      },
    })
    expect(wrapper.find('button').exists()).toBe(true)
  })
})
