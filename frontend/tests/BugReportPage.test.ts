import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import BugReportPage from '../app/pages/bug-report.vue'

describe('bug-report page', () => {
  it('renders the form with required fields, optional attachments and a honeypot', async () => {
    const wrapper = await mountSuspended(BugReportPage)

    const title = wrapper.find('input#br-title')
    const version = wrapper.find('input#br-version')
    const content = wrapper.find('textarea#br-content')
    expect(title.exists()).toBe(true)
    expect(title.attributes('required')).toBeDefined()
    expect(version.attributes('required')).toBeDefined()
    expect(content.attributes('required')).toBeDefined()

    const email = wrapper.find('input#br-email')
    expect(email.exists()).toBe(true)
    expect(email.attributes('required')).toBeUndefined()

    expect(wrapper.find('input#br-screenshot').attributes('accept')).toContain('image/')
    expect(wrapper.find('input#br-log').exists()).toBe(true)

    const honeypot = wrapper.find('input[name="website"]')
    expect(honeypot.exists()).toBe(true)
    expect(honeypot.classes()).toContain('hidden')

    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })
})
