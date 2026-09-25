<template>
  <div class="py-16 min-h-screen">
    <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-12">
        <h1 class="text-4xl font-bold text-emerald-50 mb-4">{{ $t('download.title') }}</h1>
        <p class="text-emerald-100/60 max-w-2xl mx-auto">{{ $t('download.subtitle') }}</p>
        <p v-if="stats" class="mt-3 text-sm text-gold-400/90">
          {{ $t('download.totalDownloads', { n: stats.total }) }}
        </p>
      </div>

      <div v-if="pending" class="text-center py-20 text-emerald-100/60">{{ $t('loading') }}</div>

      <div v-else-if="latest" class="mb-12">
        <div class="bg-forest-900 rounded-2xl border border-gold-500/15 shadow-lg shadow-black/30 p-8">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl font-bold text-emerald-50">{{ $t('download.latest') }} {{ latest.version }}</h2>
            <span v-if="latest.is_prerelease" class="px-3 py-1 text-sm bg-gold-500/15 text-gold-300 border border-gold-500/30 rounded-full">Pre-release</span>
          </div>

          <div v-if="latestReadyAssets.length" class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div
              v-for="asset in latestReadyAssets"
              :key="asset.id"
              class="p-4 bg-forest-950/60 border border-gold-500/10 rounded-xl flex flex-col gap-2 hover:border-gold-500/40 transition"
            >
              <div class="font-semibold text-lg text-emerald-50">
                {{ platformLabel(asset.platform) }}
                <span v-if="packageFormat(asset.filename)" class="ml-1 px-1.5 py-0.5 text-xs bg-forest-800 text-gold-300 border border-gold-500/20 rounded">{{ packageFormat(asset.filename) }}</span>
              </div>
              <div class="text-sm text-emerald-100/50">{{ asset.arch || 'x86_64' }} · {{ formatSize(asset.size_bytes) }}</div>
              <div v-if="downloadCounts[asset.id] !== undefined" class="text-xs text-gold-400/80">
                {{ $t('download.countLabel', { n: downloadCounts[asset.id] }) }}
              </div>
              <DownloadButton
                :release-id="latest.id"
                :platform="asset.platform"
                :arch="asset.arch || undefined"
                :asset-id="asset.id"
                class="mt-auto"
              >
                {{ $t('download.download') }}
              </DownloadButton>
            </div>
          </div>
          <div v-else class="text-center py-8 text-emerald-100/50 bg-forest-950/60 rounded-xl">
            {{ $t('download.noAssets') }}
          </div>

          <div v-if="latest.release_notes" class="mt-8">
            <h3 class="text-lg font-semibold text-emerald-50 mb-2">{{ $t('download.releaseNotes') }}</h3>
            <div class="prose prose-sm prose-invert max-w-none text-emerald-100/70 bg-forest-950/60 border border-gold-500/10 p-4 rounded-lg" v-html="renderedReleaseNotes" />
          </div>
        </div>
      </div>

      <div v-if="allReleases && allReleases.length > 1">
        <h2 class="text-2xl font-bold text-emerald-50 mb-6">{{ $t('download.previous') }}</h2>
        <div class="space-y-6">
          <ReleaseCard
            v-for="release in allReleases.slice(1)"
            :key="release.id"
            :release="release"
            :download-counts="downloadCounts"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DownloadStats, Release } from '~/types'

const { fetchApi } = useApi()
const { t } = useI18n()

// SSG: data is fetched in the browser at runtime, not at build time.
const { data: allReleases, pending } = await useAsyncData<Release[]>('releases', async () => {
  const res = await fetchApi<{ releases: Release[] }>('/api/v1/releases')
  return res.releases
}, {
  server: false,
})

const { data: stats } = await useAsyncData<DownloadStats | null>('download-stats', async () => {
  try {
    return await fetchApi<DownloadStats>('/api/v1/stats/downloads')
  } catch {
    return null
  }
}, {
  server: false,
})

const downloadCounts = computed<Record<string, number>>(() => {
  const map: Record<string, number> = {}
  for (const entry of stats.value?.per_asset ?? []) {
    map[entry.asset_id] = entry.count
  }
  return map
})

const latest = computed(() => allReleases.value?.[0] || null)
const latestReadyAssets = computed(() => latest.value?.assets.filter(a => a.sync_status === 'ready') || [])
const renderedReleaseNotes = computed(() => latest.value?.release_notes ? renderMarkdown(latest.value.release_notes) : '')

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
