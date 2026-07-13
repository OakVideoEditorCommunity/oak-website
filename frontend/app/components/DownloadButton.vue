<template>
  <button
    class="inline-flex items-center justify-center px-6 py-3 bg-emerald-600 text-white rounded-xl font-semibold text-lg shadow hover:bg-emerald-700 transition focus:outline-none focus:ring-2 focus:ring-emerald-500"
    :disabled="loading"
    @click="handleDownload"
  >
    <span v-if="loading">{{ $t('download.preparing') }}</span>
    <span v-else>
      <slot />
    </span>
  </button>
</template>

<script setup lang="ts">
interface Props {
  releaseId: string
  platform: string
  arch?: string
}

const props = defineProps<Props>()
const { fetchApi } = useApi()
const loading = ref(false)

async function handleDownload() {
  loading.value = true
  try {
    const params = new URLSearchParams({ platform: props.platform })
    if (props.arch) params.append('arch', props.arch)
    const url = `/api/v1/releases/${props.releaseId}/download?${params.toString()}`

    // Let the browser hit the backend download endpoint, which responds
    // with a 302 redirect to a presigned R2 URL.
    window.location.href = url
  } catch (e) {
    console.error('Download failed', e)
    alert('Download failed')
  } finally {
    loading.value = false
  }
}
</script>
