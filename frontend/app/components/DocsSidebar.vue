<template>
  <aside class="w-full md:w-64 shrink-0">
    <div class="bg-forest-900 border border-gold-500/15 rounded-xl shadow shadow-black/30 p-4 sticky top-20">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-bold text-emerald-50">{{ $t('docs.title') }}</h3>
        <NuxtLink
          :to="localePath('/docs' as any)"
          class="text-sm text-gold-400 hover:text-gold-300 transition"
        >
          {{ $t('docs.index') }}
        </NuxtLink>
      </div>

      <div class="space-y-4">
        <div v-for="lang in ['zh', 'en']" :key="lang">
          <h4 class="text-xs font-semibold text-emerald-100/50 uppercase tracking-wide mb-2">
            {{ lang === 'zh' ? '中文' : 'English' }}
          </h4>
          <ul class="space-y-1">
            <li v-for="page in pages[lang]" :key="page.slug">
              <NuxtLink
                :to="localePath(docPath(version, lang, page.slug) as any)"
                class="block text-sm px-2 py-1 rounded text-emerald-100/70 hover:bg-forest-800 hover:text-gold-300 transition"
                :class="{ 'bg-forest-800 text-gold-300 font-medium': isActive(page) }"
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
