<template>
  <div>
    <HeroSection :latest="latest" />

    <section class="py-16">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <h2 class="text-3xl font-bold text-emerald-50 mb-4">{{ $t('features.title') }}</h2>
          <p class="text-emerald-100/60 max-w-2xl mx-auto">{{ $t('features.subtitle') }}</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div
            v-for="feature in features"
            :key="feature.key"
            class="p-6 bg-forest-900 border border-gold-500/15 rounded-2xl hover:border-gold-500/40 transition"
          >
            <h3 class="text-xl font-semibold text-gold-300 mb-2">{{ $t(`features.${feature.key}.title`) }}</h3>
            <p class="text-emerald-100/70">{{ $t(`features.${feature.key}.desc`) }}</p>
          </div>
        </div>

        <div class="text-center mt-10">
          <NuxtLink :to="localePath('/features' as any)" class="text-gold-400 hover:text-gold-300 font-medium transition">
            {{ $t('features.more') }} →
          </NuxtLink>
        </div>
      </div>
    </section>

    <CommunitySection />

    <section class="py-16 bg-gradient-to-br from-forest-900 to-forest-950 border-t border-gold-500/10">
      <div class="max-w-4xl mx-auto px-4 text-center">
        <h2 class="text-3xl font-bold text-emerald-50 mb-4">{{ $t('cta.title') }}</h2>
        <p class="text-emerald-100/60 mb-8">{{ $t('cta.subtitle') }}</p>
        <NuxtLink
          :to="localePath('/download' as any)"
          class="inline-flex items-center px-8 py-4 bg-gold-500 text-forest-950 rounded-xl font-bold text-lg shadow-lg shadow-gold-500/20 hover:bg-gold-400 transition"
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

// SSG: data is fetched in the browser at runtime, not at build time.
const { data: latest } = await useAsyncData<Release | null>('latest-release', async () => {
  try {
    return await fetchApi<Release>('/api/v1/releases/latest')
  } catch {
    return null
  }
}, {
  server: false,
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
