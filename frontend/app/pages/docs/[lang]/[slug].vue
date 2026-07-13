<template>
  <div class="py-12 bg-gray-50 min-h-screen">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row gap-8">
      <DocsSidebar :pages="docsIndex || { zh: [], en: [] }" :current-lang="lang" :current-slug="slug" />

      <article class="flex-1 bg-white rounded-2xl shadow p-8">
        <div v-if="pending" class="py-20 text-center">{{ $t('loading') }}</div>
        <div v-else-if="doc">
          <h1 class="text-3xl font-bold mb-6">{{ doc.title }}</h1>
          <div class="prose prose-emerald max-w-none doc-content" v-html="sanitizedHtml" />
        </div>
        <div v-else class="py-20 text-center text-gray-500">{{ $t('docs.notFound') }}</div>
      </article>
    </div>
  </div>
</template>

<script setup lang="ts">
import DOMPurify from 'isomorphic-dompurify'
import type { DocPage, DocsIndex } from '~/types'

const route = useRoute()
const { fetchApi } = useApi()
const { t } = useI18n()

const lang = computed(() => route.params.lang as string)
const slug = computed(() => route.params.slug as string)

const { data: docsIndex } = await useAsyncData<DocsIndex>('docs-index', async () => {
  return await fetchApi<DocsIndex>('/api/v1/docs')
})

const { data: doc, pending } = await useAsyncData<DocPage | null>(
  () => `doc-${lang.value}-${slug.value}`,
  async () => {
    try {
      return await fetchApi<DocPage>(`/api/v1/docs/${encodeURIComponent(lang.value)}/${encodeURIComponent(slug.value)}`)
    } catch {
      return null
    }
  },
  {
    watch: [lang, slug],
  }
)

const sanitizedHtml = computed(() => {
  if (!doc.value?.html) return ''
  return DOMPurify.sanitize(doc.value.html, {
    ALLOWED_TAGS: [
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'br', 'hr',
      'ul', 'ol', 'li', 'strong', 'em', 'a', 'img', 'code', 'pre',
      'table', 'thead', 'tbody', 'tr', 'th', 'td', 'blockquote', 'div', 'span',
    ],
    ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class', 'id'],
  })
})

useOakSeo({
  title: doc.value?.title || t('seo.docsTitle'),
  description: t('seo.docsDescription'),
})
</script>

<style scoped>
.doc-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
}
.doc-content :deep(pre) {
  background: #f3f4f6;
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
}
.doc-content :deep(code) {
  background: #f3f4f6;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
}
</style>
