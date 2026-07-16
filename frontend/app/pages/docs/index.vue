<template>
  <div class="py-16 bg-gray-50 min-h-screen">
    <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-12">
        <h1 class="text-4xl font-bold text-gray-900 mb-4">{{ $t('docs.title') }}</h1>
        <p class="text-gray-600">{{ $t('docs.subtitle') }}</p>
        <div class="mt-4 flex justify-center">
          <DocsVersionSwitcher :version="null" />
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div class="bg-white rounded-2xl shadow p-6">
          <h2 class="text-2xl font-bold mb-4">中文文档</h2>
          <ul class="space-y-2">
            <li v-for="page in docsIndex?.zh" :key="page.slug">
              <NuxtLink
                :to="localePath(`/docs/zh/${page.slug}` as any)"
                class="text-emerald-600 hover:underline"
              >
                {{ page.title }}
              </NuxtLink>
            </li>
          </ul>
        </div>

        <div class="bg-white rounded-2xl shadow p-6">
          <h2 class="text-2xl font-bold mb-4">English Docs</h2>
          <ul class="space-y-2">
            <li v-for="page in docsIndex?.en" :key="page.slug">
              <NuxtLink
                :to="localePath(`/docs/en/${page.slug}` as any)"
                class="text-emerald-600 hover:underline"
              >
                {{ page.title }}
              </NuxtLink>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DocsIndex } from '~/types'

const { fetchApi } = useApi()
const { t } = useI18n()
const localePath = useLocalePath()

const { data: docsIndex } = await useAsyncData<DocsIndex>('docs-index', async () => {
  return await fetchApi<DocsIndex>('/api/v1/docs')
})

useOakSeo({
  title: t('seo.docsTitle'),
  description: t('seo.docsDescription'),
})
</script>
