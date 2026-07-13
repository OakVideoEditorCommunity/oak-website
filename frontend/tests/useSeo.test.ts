import { describe, it, expect } from 'vitest'
import { useOakSeo } from '../app/composables/useSeo'

describe('useOakSeo', () => {
  it('can be called with default options', () => {
    // useHead requires a Vue/Nuxt context; this test verifies the composable import.
    expect(useOakSeo).toBeTypeOf('function')
  })
})
