<template>
  <div class="py-16 min-h-screen">
    <div class="max-w-md mx-auto px-4 sm:px-6 lg:px-8">
      <div class="bg-forest-900 rounded-2xl border border-gold-500/15 shadow-lg shadow-black/30 p-8">
        <h1 class="text-2xl font-bold text-emerald-50 mb-2">Admin</h1>
        <p class="text-emerald-100/60 mb-6">Trigger a GitHub → R2 release sync.</p>

        <form class="space-y-4" @submit.prevent="handleSync">
          <div>
            <label for="admin-token" class="block text-sm font-medium text-emerald-100/80 mb-1">
              Admin Token
            </label>
            <input
              id="admin-token"
              v-model="token"
              type="password"
              required
              class="w-full px-4 py-2 bg-forest-950/60 border border-gold-500/20 rounded-lg text-emerald-50 placeholder-emerald-100/30 focus:ring-2 focus:ring-gold-400 focus:border-gold-400 outline-none"
              placeholder="Bearer token"
            />
          </div>

          <button
            type="submit"
            :disabled="loading"
            class="w-full px-4 py-2 bg-gold-500 text-forest-950 rounded-lg font-semibold hover:bg-gold-400 transition disabled:opacity-50"
          >
            <span v-if="loading">Syncing…</span>
            <span v-else>Sync Releases</span>
          </button>
        </form>

        <div
          v-if="message"
          :class="success ? 'bg-forest-800 text-emerald-200 border border-emerald-500/30' : 'bg-red-950/60 text-red-300 border border-red-500/30'"
          class="mt-4 p-3 rounded-lg text-sm"
        >
          {{ message }}
        </div>
      </div>

      <div v-if="token" class="bg-forest-900 rounded-2xl border border-gold-500/15 shadow-lg shadow-black/30 p-8 mt-8">
        <h2 class="text-xl font-bold text-emerald-50 mb-2">Bug Reports</h2>
        <p class="text-emerald-100/60 mb-6">View submitted bug reports (uses the token above).</p>

        <button
          :disabled="!token || reportsLoading"
          class="w-full px-4 py-2 border border-gold-500/40 text-gold-300 rounded-lg font-semibold hover:bg-gold-500/10 transition disabled:opacity-50"
          @click="loadReports"
        >
          <span v-if="reportsLoading">Loading…</span>
          <span v-else>Load Bug Reports</span>
        </button>

        <div
          v-if="reportsError"
          class="mt-4 p-3 rounded-lg text-sm bg-red-950/60 text-red-300 border border-red-500/30"
        >
          {{ reportsError }}
        </div>

        <p v-else-if="reportsLoaded && reports.length === 0" class="mt-4 text-sm text-emerald-100/50">
          No bug reports yet.
        </p>

        <ul v-else-if="reports.length" class="mt-6 space-y-4 text-left">
          <li
            v-for="report in reports"
            :key="report.id"
            class="bg-forest-950/60 border border-gold-500/10 rounded-xl p-4"
          >
            <div class="flex items-start justify-between gap-3 mb-1">
              <h3 class="font-semibold text-emerald-50">{{ report.title }}</h3>
              <span class="shrink-0 px-2 py-0.5 text-xs bg-forest-800 text-gold-300 border border-gold-500/20 rounded">
                {{ report.app_version }}
              </span>
            </div>
            <p class="text-xs text-emerald-100/40 mb-2">
              {{ formatDate(report.created_at) }}
              <template v-if="report.email"> · <a :href="`mailto:${report.email}`" class="text-gold-400 hover:text-gold-300">{{ report.email }}</a></template>
            </p>
            <p class="text-sm text-emerald-100/70 whitespace-pre-wrap">{{ report.content }}</p>
            <div v-if="report.screenshot_url || report.log_url" class="mt-2 flex gap-4 text-sm">
              <a
                v-if="report.screenshot_url"
                :href="report.screenshot_url"
                target="_blank"
                rel="noopener"
                class="text-gold-400 hover:text-gold-300"
              >Screenshot<template v-if="report.screenshot_filename"> ({{ report.screenshot_filename }})</template></a>
              <a
                v-if="report.log_url"
                :href="report.log_url"
                target="_blank"
                rel="noopener"
                class="text-gold-400 hover:text-gold-300"
              >Log<template v-if="report.log_filename"> ({{ report.log_filename }})</template></a>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface BugReport {
  id: string
  title: string
  app_version: string
  content: string
  email: string | null
  screenshot_filename: string | null
  screenshot_url: string | null
  log_filename: string | null
  log_url: string | null
  created_at: string
}

const { fetchApi } = useApi()

const token = ref('')
const loading = ref(false)
const message = ref('')
const success = ref(false)

const reports = ref<BugReport[]>([])
const reportsLoading = ref(false)
const reportsLoaded = ref(false)
const reportsError = ref('')

// A different token invalidates whatever the previous one loaded.
watch(token, () => {
  reports.value = []
  reportsLoaded.value = false
  reportsError.value = ''
})

async function handleSync() {
  loading.value = true
  message.value = ''
  success.value = false

  try {
    // 202: the sync runs in the background (it can take minutes); poll the
    // status endpoint until it finishes.
    await fetchApi<{ synced: number; message: string }>('/api/admin/releases/sync', {
      method: 'POST',
      body: { tag: undefined },
      headers: {
        Authorization: `Bearer ${token.value}`,
      },
    })
    message.value = 'Sync started in the background…'

    const deadline = Date.now() + 15 * 60 * 1000
    while (Date.now() < deadline) {
      await new Promise(resolve => setTimeout(resolve, 3000))
      const status = await fetchApi<{ running: boolean; last_result: string | null }>(
        '/api/admin/releases/sync/status',
        { headers: { Authorization: `Bearer ${token.value}` } },
      )
      if (!status.running) {
        message.value = status.last_result || 'Sync finished.'
        success.value = !(status.last_result || '').startsWith('failed')
        return
      }
    }
    message.value = 'Sync is still running in the background; check back later.'
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

async function loadReports() {
  reportsLoading.value = true
  reportsError.value = ''

  try {
    const res = await fetchApi<{ reports: BugReport[] }>('/api/admin/bug-reports', {
      headers: {
        Authorization: `Bearer ${token.value}`,
      },
    })
    reports.value = res.reports
    reportsLoaded.value = true
  } catch (e: any) {
    if (e?.status === 401) {
      reportsError.value = 'Unauthorized: invalid admin token.'
    } else {
      reportsError.value = e?.message || 'Failed to load bug reports.'
    }
  } finally {
    reportsLoading.value = false
  }
}

function formatDate(date: string) {
  return new Date(date).toLocaleString()
}

useOakSeo({
  title: 'Admin',
})
</script>
