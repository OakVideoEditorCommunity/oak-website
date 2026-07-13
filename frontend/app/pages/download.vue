<template>
  <div class="py-16 bg-gray-50 min-h-screen">
    <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-12">
        <h1 class="text-4xl font-bold text-gray-900 mb-4">{{ $t('download.title') }}</h1>
        <p class="text-gray-600 max-w-2xl mx-auto">{{ $t('download.subtitle') }}</p>
      </div>

      <div v-if="pending" class="text-center py-20">{{ $t('loading') }}</div>

      <div v-else-if="latest" class="mb-12">
        <div class="bg-white rounded-2xl shadow-lg p-8 border border-emerald-100">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl font-bold">{{ $t('download.latest') }} {{ latest.version }}</h2>
            <span v-if="latest.is_prerelease" class="px-3 py-1 text-sm bg-amber-100 text-amber-800 rounded-full">Pre-release</span>
          </div>

          <div v-if="latestReadyAssets.length" class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div
              v-for="asset in latestReadyAssets"
              :key="asset.id"
              class="p-4 border rounded-xl flex flex-col gap-2"
            >
              <div class="font-semibold text-lg">{{ platformLabel(asset.platform) }}</div>
              <div class="text-sm text-gray-500">{{ asset.arch || 'x86_64' }} · {{ formatSize(asset.size_bytes) }}</div>
              <DownloadButton
                :release-id="latest.id"
                :platform="asset.platform"
                :arch="asset.arch || undefined"
                class="mt-auto"
              >
                {{ $t('download.download') }}
              </DownloadButton>
            </div>
          </div>
          <div v-else class="text-center py-8 text-gray-500 bg-gray-50 rounded-xl">
            {{ $t('download.noAssets') }}
          </div>

          <div v-if="latest.release_notes" class="mt-8 prose max-w-none">
            <h3 class="text-lg font-semibold mb-2">{{ $t('download.releaseNotes') }}</h3>
            <pre class="whitespace-pre-wrap text-sm text-gray-700 bg-gray-50 p-4 rounded-lg">{{ latest.release_notes }}</pre>
          </div>
        </div>
      </div>

      <div v-if="allReleases && allReleases.length > 1">
        <h2 class="text-2xl font-bold text-gray-900 mb-6">{{ $t('download.previous') }}</h2>
        <div class="space-y-6">
          <ReleaseCard
            v-for="release in allReleases.slice(1)"
            :key="release.id"
            :release="release"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Release } from '~/types'

const { fetchApi } = useApi()
const { t } = useI18n()

const { data: allReleases, pending } = await useAsyncData<Release[]>('releases', async () => {
  const res = await fetchApi<{ releases: Release[] }>('/api/v1/releases')
  return res.releases
})

const latest = computed(() => allReleases.value?.[0] || null)
const latestReadyAssets = computed(() => latest.value?.assets.filter(a => a.sync_status === 'ready') || [])

function platformLabel(platform: string) {
  const labels: Record<string, string> = {
    windows: t('platform.windows'),
    macos: t('platform.macos'),
    linux: t('platform.linux'),
    unknown: t('platform.unknown'),
  }
  return labels[platform] || platform
}

function formatSize(bytes?: number | null) {
  if (!bytes) return '-'
  const mb = bytes / 1024 / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

useOakSeo({
  title: t('seo.downloadTitle'),
  description: t('seo.downloadDescription'),
})
</script>
