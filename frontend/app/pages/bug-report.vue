<template>
  <div class="py-16 min-h-screen">
    <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-10">
        <h1 class="text-4xl font-bold text-emerald-50 mb-4">{{ $t('bugReport.title') }}</h1>
        <p class="text-emerald-100/60">{{ $t('bugReport.subtitle') }}</p>
      </div>

      <div class="bg-forest-900 border border-gold-500/15 rounded-2xl shadow-lg shadow-black/30 p-8">
        <div v-if="submitted" class="text-center py-8">
          <p class="text-xl font-semibold text-gold-300 mb-2">{{ $t('bugReport.success') }}</p>
          <p class="text-sm text-emerald-100/50 mb-6">{{ $t('bugReport.successHint') }}</p>
          <button
            class="px-6 py-2 border border-gold-500/40 text-gold-300 rounded-lg font-medium hover:bg-gold-500/10 transition"
            @click="reset"
          >
            {{ $t('bugReport.another') }}
          </button>
        </div>

        <form v-else class="space-y-5" @submit.prevent="handleSubmit">
          <div>
            <label for="br-title" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.title') }} <span class="text-gold-400">*</span>
            </label>
            <input
              id="br-title"
              v-model="title"
              type="text"
              required
              maxlength="200"
              class="w-full px-4 py-2 bg-forest-950/60 border border-gold-500/20 rounded-lg text-emerald-50 placeholder-emerald-100/30 focus:ring-2 focus:ring-gold-400 focus:border-gold-400 outline-none"
            />
          </div>

          <div>
            <label for="br-version" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.version') }} <span class="text-gold-400">*</span>
            </label>
            <input
              id="br-version"
              v-model="version"
              type="text"
              required
              maxlength="100"
              :placeholder="$t('bugReport.form.versionPlaceholder')"
              class="w-full px-4 py-2 bg-forest-950/60 border border-gold-500/20 rounded-lg text-emerald-50 placeholder-emerald-100/30 focus:ring-2 focus:ring-gold-400 focus:border-gold-400 outline-none"
            />
          </div>

          <div>
            <label for="br-content" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.content') }} <span class="text-gold-400">*</span>
            </label>
            <textarea
              id="br-content"
              v-model="content"
              required
              rows="6"
              :placeholder="$t('bugReport.form.contentPlaceholder')"
              class="w-full px-4 py-2 bg-forest-950/60 border border-gold-500/20 rounded-lg text-emerald-50 placeholder-emerald-100/30 focus:ring-2 focus:ring-gold-400 focus:border-gold-400 outline-none resize-y"
            />
          </div>

          <div>
            <label for="br-email" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.email') }}
            </label>
            <input
              id="br-email"
              v-model="email"
              type="email"
              maxlength="254"
              class="w-full px-4 py-2 bg-forest-950/60 border border-gold-500/20 rounded-lg text-emerald-50 placeholder-emerald-100/30 focus:ring-2 focus:ring-gold-400 focus:border-gold-400 outline-none"
            />
            <p class="mt-1 text-xs text-emerald-100/40">{{ $t('bugReport.form.emailHint') }}</p>
          </div>

          <div>
            <label for="br-screenshot" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.screenshot') }}
            </label>
            <input
              id="br-screenshot"
              type="file"
              accept="image/*"
              class="w-full text-sm text-emerald-100/60 file:bg-gold-500 file:text-forest-950 file:rounded-lg file:border-0 file:px-3 file:py-1.5 file:mr-3 file:font-medium file:cursor-pointer hover:file:bg-gold-400"
              @change="onFileChange($event, 'screenshot')"
            />
          </div>

          <div>
            <label for="br-log" class="block text-sm font-medium text-emerald-100/80 mb-1">
              {{ $t('bugReport.form.log') }}
            </label>
            <input
              id="br-log"
              type="file"
              accept=".txt,.log,text/plain"
              class="w-full text-sm text-emerald-100/60 file:bg-gold-500 file:text-forest-950 file:rounded-lg file:border-0 file:px-3 file:py-1.5 file:mr-3 file:font-medium file:cursor-pointer hover:file:bg-gold-400"
              @change="onFileChange($event, 'log')"
            />
            <p class="mt-1 text-xs text-emerald-100/40">{{ $t('bugReport.form.logHint') }}</p>
          </div>

          <!-- Honeypot: invisible to humans, bots fill it and get a fake success. -->
          <input
            v-model="website"
            type="text"
            name="website"
            tabindex="-1"
            autocomplete="off"
            class="hidden"
            aria-hidden="true"
          />

          <button
            type="submit"
            :disabled="submitting"
            class="w-full px-4 py-3 bg-gold-500 text-forest-950 rounded-lg font-semibold hover:bg-gold-400 transition disabled:opacity-50"
          >
            {{ submitting ? $t('bugReport.form.submitting') : $t('bugReport.form.submit') }}
          </button>

          <div v-if="error" class="p-3 rounded-lg text-sm bg-red-950/60 text-red-300 border border-red-500/30">
            {{ error }}
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { fetchApi } = useApi()
const { t } = useI18n()

const title = ref('')
const version = ref('')
const content = ref('')
const email = ref('')
const website = ref('')
const screenshot = ref<File | null>(null)
const log = ref<File | null>(null)
const submitting = ref(false)
const submitted = ref(false)
const error = ref('')

function onFileChange(event: Event, kind: 'screenshot' | 'log') {
  const file = (event.target as HTMLInputElement).files?.[0] || null
  if (kind === 'screenshot') {
    screenshot.value = file
  } else {
    log.value = file
  }
}

async function handleSubmit() {
  submitting.value = true
  error.value = ''

  const form = new FormData()
  form.append('title', title.value)
  form.append('version', version.value)
  form.append('content', content.value)
  if (email.value) form.append('email', email.value)
  if (website.value) form.append('website', website.value)
  if (screenshot.value) form.append('screenshot', screenshot.value)
  if (log.value) form.append('log', log.value)

  try {
    await fetchApi('/api/v1/bug-reports', { method: 'POST', body: form })
    submitted.value = true
  } catch (e: any) {
    error.value = e?.data?.error || t('bugReport.error')
  } finally {
    submitting.value = false
  }
}

function reset() {
  title.value = ''
  version.value = ''
  content.value = ''
  email.value = ''
  website.value = ''
  screenshot.value = null
  log.value = null
  error.value = ''
  submitted.value = false
}

useOakSeo({
  title: t('seo.bugReportTitle'),
  description: t('seo.bugReportDescription'),
})
</script>
