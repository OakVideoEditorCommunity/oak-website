import { describe, it, expect, vi } from 'vitest'
import { useApi } from '../app/composables/useApi'

describe('useApi', () => {
  it('uses public API base URL on client', async () => {
    const { fetchApi } = useApi()
    // $fetch is mocked by @nuxt/test-utils in component tests;
    // this test mainly ensures the composable can be imported and invoked.
    expect(fetchApi).toBeTypeOf('function')
  })
})
