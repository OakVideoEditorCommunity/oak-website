<template>
  <div class="py-16 min-h-screen">
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
      <div v-if="release">
        <ReleaseCard :release="release" />
      </div>
      <div v-else class="text-center text-emerald-100/50">{{ $t('releases.notFound') }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Release } from '~/types'

const route = useRoute()
const { fetchApi } = useApi()
const { t } = useI18n()

const version = computed(() => route.params.version as string)

// SSG: data is fetched in the browser at runtime, not at build time.
const { data: release } = await useAsyncData<Release | null>(
  () => `release-${version.value}`,
  async () => {
    try {
      const all = await fetchApi<{ releases: Release[] }>('/api/v1/releases')
      return all.releases.find(r => r.version === version.value || r.tag_name === version.value) || null
    } catch {
      return null
    }
  },
  {
    watch: [version],
    server: false,
  }
)

useOakSeo({
  title: release.value?.version || t('seo.downloadTitle'),
  description: t('seo.downloadDescription'),
})
</script>
