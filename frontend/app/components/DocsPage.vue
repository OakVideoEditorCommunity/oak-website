<template>
  <div class="py-12 bg-gray-50 min-h-screen">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row gap-8">
      <div class="w-full md:w-64 shrink-0">
        <div class="bg-white rounded-xl shadow p-4 mb-4">
          <DocsVersionSwitcher :version="version" :lang="lang" :slug="slug" />
        </div>
        <DocsSidebar
          :pages="docsIndex || { version: '', zh: [], en: [] }"
          :current-lang="lang"
          :current-slug="slug"
          :version="version"
        />
      </div>

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

interface Props {
  // null = default (latest) version; otherwise a concrete pinned version.
  version?: string | null
}

const props = withDefaults(defineProps<Props>(), { version: null })

const route = useRoute()
const { fetchApi } = useApi()
const { t } = useI18n()
const localePath = useLocalePath()

const lang = computed(() => route.params.lang as string)
const slug = computed(() => route.params.slug as string)

const { data: docsIndex } = await useAsyncData<DocsIndex>(
  () => `docs-index-${props.version ?? 'latest'}`,
  async () => {
    try {
      return await fetchApi<DocsIndex>(
        props.version
          ? `/api/v1/docs?version=${encodeURIComponent(props.version)}`
          : '/api/v1/docs'
      )
    } catch {
      return { version: '', zh: [], en: [] }
    }
  },
  {
    watch: [() => props.version],
  }
)

const { data: doc, pending } = await useAsyncData<DocPage | null>(
  () => `doc-${props.version ?? 'latest'}-${lang.value}-${slug.value}`,
  async () => {
    try {
      const url = props.version
        ? `/api/v1/docs/${encodeURIComponent(props.version)}/${encodeURIComponent(lang.value)}/${encodeURIComponent(slug.value)}`
        : `/api/v1/docs/${encodeURIComponent(lang.value)}/${encodeURIComponent(slug.value)}`
      return await fetchApi<DocPage>(url)
    } catch {
      return null
    }
  },
  {
    watch: [lang, slug, () => props.version],
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
  // Pinned-version pages canonicalize to the default-version URL.
  canonicalPath: props.version
    ? localePath(`/docs/${lang.value}/${slug.value}` as any)
    : undefined,
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
