<template>
  <div>
    <HeroSection :latest="latest" />

    <section class="py-16 bg-white">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <h2 class="text-3xl font-bold text-gray-900 mb-4">{{ $t('features.title') }}</h2>
          <p class="text-gray-600 max-w-2xl mx-auto">{{ $t('features.subtitle') }}</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div v-for="feature in features" :key="feature.key" class="p-6 bg-gray-50 rounded-2xl">
            <h3 class="text-xl font-semibold mb-2">{{ $t(`features.${feature.key}.title`) }}</h3>
            <p class="text-gray-600">{{ $t(`features.${feature.key}.desc`) }}</p>
          </div>
        </div>
      </div>
    </section>

    <section class="py-16 bg-emerald-50">
      <div class="max-w-4xl mx-auto px-4 text-center">
        <h2 class="text-3xl font-bold text-gray-900 mb-4">{{ $t('cta.title') }}</h2>
        <p class="text-gray-600 mb-8">{{ $t('cta.subtitle') }}</p>
        <NuxtLink
          :to="localePath('/download' as any)"
          class="inline-flex items-center px-8 py-4 bg-emerald-600 text-white rounded-xl font-bold text-lg shadow hover:bg-emerald-700 transition"
        >
          {{ $t('cta.button') }}
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { Release } from '~/types'

const { fetchApi } = useApi()
const localePath = useLocalePath()

const { data: latest } = await useAsyncData<Release | null>('latest-release', async () => {
  try {
    return await fetchApi<Release>('/api/v1/releases/latest')
  } catch {
    return null
  }
})

const features = [
  { key: 'nonlinear' },
  { key: 'color' },
  { key: 'openfx' },
]

useOakSeo({
  title: useI18n().t('seo.homeTitle'),
  description: useI18n().t('seo.defaultDescription'),
  jsonLd: {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: 'Oak Video Editor',
    applicationCategory: 'VideoEditor',
    operatingSystem: 'Windows, macOS, Linux',
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
    },
  },
})
</script>
