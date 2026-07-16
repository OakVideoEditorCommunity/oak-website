<template>
  <div v-if="versions.length" class="flex items-center gap-2">
    <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
      {{ $t('docs.version') }}
    </label>
    <select
      :value="version ?? latest"
      class="bg-gray-100 border border-gray-300 text-gray-700 text-sm rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-emerald-500"
      @change="switchVersion"
    >
      <option v-for="v in versions" :key="v" :value="v">
        {{ v === latest ? `${v} (${$t('docs.latest')})` : v }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import type { DocsIndex } from '~/types'

interface Props {
  // null = default (latest) version.
  version?: string | null
  lang?: string
  slug?: string
}

const props = withDefaults(defineProps<Props>(), { version: null })

const { fetchApi } = useApi()
const { locale } = useI18n()
const localePath = useLocalePath()
const { versions, latest, docPath } = useDocsVersions()

async function switchVersion(event: Event) {
  const target = (event.target as HTMLSelectElement).value

  let toc: DocsIndex | null = null
  try {
    toc = await fetchApi<DocsIndex>(
      target === latest.value
        ? '/api/v1/docs'
        : `/api/v1/docs?version=${encodeURIComponent(target)}`
    )
  } catch {
    toc = null
  }

  // Keep the reader on the same doc when it exists in the target version;
  // otherwise land on the first doc of the current (or other) language.
  const preferredLang = props.lang || (locale.value === 'zh' ? 'zh' : 'en')
  let path = '/docs'
  if (toc) {
    const pages = preferredLang === 'zh' ? toc.zh : toc.en
    const otherPages = preferredLang === 'zh' ? toc.en : toc.zh
    const match = props.slug ? pages.find((p) => p.slug === props.slug) : undefined
    const first = match || pages[0] || otherPages[0]
    if (first) {
      path = docPath(target, first.lang, first.slug)
    }
  }
  await navigateTo(localePath(path as any))
}
</script>
