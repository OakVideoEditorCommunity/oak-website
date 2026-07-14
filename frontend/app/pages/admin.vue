<template>
  <div class="py-16 bg-gray-50 min-h-screen">
    <div class="max-w-md mx-auto px-4 sm:px-6 lg:px-8">
      <div class="bg-white rounded-2xl shadow p-8">
        <h1 class="text-2xl font-bold text-gray-900 mb-2">Admin</h1>
        <p class="text-gray-600 mb-6">Trigger a GitHub → R2 release sync.</p>

        <form class="space-y-4" @submit.prevent="handleSync">
          <div>
            <label for="admin-token" class="block text-sm font-medium text-gray-700 mb-1">
              Admin Token
            </label>
            <input
              id="admin-token"
              v-model="token"
              type="password"
              required
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 outline-none"
              placeholder="Bearer token"
            />
          </div>

          <button
            type="submit"
            :disabled="loading"
            class="w-full px-4 py-2 bg-emerald-600 text-white rounded-lg font-medium hover:bg-emerald-700 transition disabled:opacity-50"
          >
            <span v-if="loading">Syncing…</span>
            <span v-else>Sync Releases</span>
          </button>
        </form>

        <div
          v-if="message"
          :class="success ? 'bg-emerald-50 text-emerald-800' : 'bg-red-50 text-red-800'"
          class="mt-4 p-3 rounded-lg text-sm"
        >
          {{ message }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { fetchApi } = useApi()

const token = ref('')
const loading = ref(false)
const message = ref('')
const success = ref(false)

async function handleSync() {
  loading.value = true
  message.value = ''
  success.value = false

  try {
    const res = await fetchApi<{ synced: number; message: string }>('/api/admin/releases/sync', {
      method: 'POST',
      body: { tag: undefined },
      headers: {
        Authorization: `Bearer ${token.value}`,
      },
    })
    message.value = `Synced ${res.synced} asset(s).`
    success.value = true
  } catch (e: any) {
    if (e?.status === 401) {
      message.value = 'Unauthorized: invalid admin token.'
    } else {
      message.value = e?.message || 'Sync failed. Check the server logs.'
    }
  } finally {
    loading.value = false
  }
}

useOakSeo({
  title: 'Admin',
})
</script>
