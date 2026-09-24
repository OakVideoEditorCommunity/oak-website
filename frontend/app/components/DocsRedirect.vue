<template>
  <div class="py-24 min-h-screen">
    <div class="max-w-xl mx-auto px-4 text-center">
      <h1 class="text-3xl font-bold text-emerald-50 mb-4">{{ $t('docs.movedTitle') }}</h1>
      <p class="text-emerald-100/60 mb-8">{{ $t('docs.moved') }}</p>
      <a
        :href="target"
        rel="noopener"
        class="inline-flex items-center px-6 py-3 bg-gold-500 text-forest-950 rounded-xl font-semibold shadow-lg shadow-gold-500/20 hover:bg-gold-400 transition"
      >
        {{ $t('docs.movedButton') }}
      </a>
    </div>
  </div>
</template>

<script setup lang="ts">
// The documentation lives on https://docs.oakvideoeditor.org now. The old
// in-site /docs routes are kept as redirect stubs: a meta refresh plus a
// JS replace send the browser to the matching page on the docs site, and
// the link above covers clients with JavaScript disabled. The frontend
// nginx also answers /docs/* with a real 301 for direct hits.
const DOCS_ORIGIN = 'https://docs.oakvideoeditor.org'

const route = useRoute()
const { t } = useI18n()

// Strip the i18n locale prefix (only present when followed by /docs) and the
// /docs segment itself; the rest of the path is preserved on the docs site
// (e.g. /docs/zh/quick-start -> .../zh/quick-start).
const target = computed(() => {
  const path = route.path.replace(/^\/[a-z]{2}(?=\/docs)/, '').replace(/^\/docs/, '')
  return `${DOCS_ORIGIN}${path || '/'}`
})

useHead({
  meta: [
    { 'http-equiv': 'refresh', content: `0;url=${target.value}` },
  ],
})

onMounted(() => {
  window.location.replace(target.value)
})

useOakSeo({
  title: t('seo.docsTitle'),
  description: t('seo.docsDescription'),
})
</script>
