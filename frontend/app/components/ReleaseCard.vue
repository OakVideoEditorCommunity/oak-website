<template>
  <div class="bg-forest-900 rounded-2xl border border-gold-500/15 shadow shadow-black/30 p-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-xl font-bold text-emerald-50">{{ release.version }}</h3>
      <span v-if="release.is_prerelease" class="px-2 py-1 text-xs font-medium bg-gold-500/15 text-gold-300 border border-gold-500/30 rounded-full">Pre-release</span>
    </div>

    <p v-if="release.published_at" class="text-sm text-emerald-100/50 mb-4">
      {{ formatDate(release.published_at) }}
    </p>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      <div
        v-for="asset in readyAssets"
        :key="asset.id"
        class="flex flex-col gap-2 p-3 bg-forest-950/60 border border-gold-500/10 rounded-lg hover:border-gold-500/40 transition"
      >
        <div class="font-medium text-emerald-100/90">
          {{ platformLabel(asset.platform) }}
          <span v-if="packageFormat(asset.filename)" class="ml-1 px-1.5 py-0.5 text-xs bg-forest-800 text-gold-300 border border-gold-500/20 rounded">{{ packageFormat(asset.filename) }}</span>
        </div>
        <div class="text-xs text-emerald-100/50">{{ asset.arch || 'x86_64' }} · {{ formatSize(asset.size_bytes) }}</div>
        <DownloadButton
          :release-id="release.id"
          :platform="asset.platform"
          :arch="asset.arch || undefined"
          :asset-id="asset.id"
          class="w-full text-sm py-2"
        >
          {{ $t('download.download') }}
        </DownloadButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Release } from '~/types'

interface Props {
  release: Release
}

const props = defineProps<Props>()
const { t } = useI18n()

const readyAssets = computed(() => props.release.assets.filter(a => a.sync_status === 'ready'))

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

function formatDate(date: string) {
  return new Date(date).toLocaleDateString()
}
</script>
