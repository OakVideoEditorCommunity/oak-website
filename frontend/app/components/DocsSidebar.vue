<template>
  <aside class="w-full md:w-64 shrink-0">
    <div class="bg-white rounded-xl shadow p-4 sticky top-20">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-bold text-gray-800">{{ $t('docs.title') }}</h3>
        <NuxtLink
          :to="localePath('/docs' as any)"
          class="text-sm text-emerald-600 hover:underline"
        >
          {{ $t('docs.index') }}
        </NuxtLink>
      </div>

      <div class="space-y-4">
        <div v-for="lang in ['zh', 'en']" :key="lang">
          <h4 class="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">
            {{ lang === 'zh' ? '中文' : 'English' }}
          </h4>
          <ul class="space-y-1">
            <li v-for="page in pages[lang]" :key="page.slug">
              <NuxtLink
                :to="localePath(docPath(version, lang, page.slug) as any)"
                class="block text-sm px-2 py-1 rounded hover:bg-emerald-50"
                :class="{ 'bg-emerald-100 text-emerald-800 font-medium': isActive(page) }"
              >
                {{ page.title }}
              </NuxtLink>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { DocPageSummary, DocsIndex } from '~/types'

interface Props {
  pages: DocsIndex
  currentLang?: string
  currentSlug?: string
  // null = default (latest) version; links then omit the version segment.
  version?: string | null
}

const props = withDefaults(defineProps<Props>(), { version: null })
const localePath = useLocalePath()
const { docPath } = useDocsVersions()

function isActive(page: DocPageSummary) {
  return page.lang === props.currentLang && page.slug === props.currentSlug
}
</script>
