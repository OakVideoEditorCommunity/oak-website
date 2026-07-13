import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import LanguageSwitcher from '../app/components/LanguageSwitcher.vue'

describe('LanguageSwitcher', () => {
  it('renders language options', async () => {
    const wrapper = await mountSuspended(LanguageSwitcher)
    expect(wrapper.find('select').exists()).toBe(true)
    const options = wrapper.findAll('option')
    expect(options.length).toBeGreaterThanOrEqual(2)
  })
})
