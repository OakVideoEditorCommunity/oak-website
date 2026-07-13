<template>
  <section class="relative bg-gradient-to-br from-emerald-900 to-gray-900 text-white py-24 lg:py-32">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
      <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight mb-6">
        {{ $t('hero.title') }}
      </h1>
      <p class="text-lg sm:text-xl text-emerald-100 max-w-3xl mx-auto mb-10">
        {{ $t('hero.subtitle') }}
      </p>

      <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
        <NuxtLink
          v-if="latest"
          :to="localePath('/download' as any)"
          class="inline-flex items-center px-8 py-4 bg-emerald-500 text-white rounded-xl font-bold text-lg shadow-lg hover:bg-emerald-400 transition"
        >
          {{ $t('hero.download') }}
          <span v-if="latest.version" class="ml-2 text-sm font-normal opacity-90">{{ latest.version }}</span>
        </NuxtLink>
        <NuxtLink
          :to="localePath('/docs' as any)"
          class="inline-flex items-center px-8 py-4 bg-white/10 text-white rounded-xl font-semibold text-lg hover:bg-white/20 transition"
        >
          {{ $t('hero.docs') }}
        </NuxtLink>
      </div>

      <p class="mt-4 text-sm text-emerald-200/70">
        {{ $t('hero.platforms') }}
      </p>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { Release } from '~/types'

interface Props {
  latest?: Release | null
}

defineProps<Props>()
const localePath = useLocalePath()
</script>
