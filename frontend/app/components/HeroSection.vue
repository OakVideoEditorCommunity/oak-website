<template>
  <section class="relative overflow-hidden bg-gradient-to-br from-forest-800 via-forest-950 to-black text-white py-24 lg:py-32">
    <div class="absolute -top-32 left-1/2 -translate-x-1/2 w-[42rem] h-[42rem] rounded-full bg-gold-500/10 blur-3xl pointer-events-none" />
    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
      <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight mb-6 text-transparent bg-clip-text bg-gradient-to-r from-gold-300 via-gold-400 to-gold-500">
        {{ $t('hero.title') }}
      </h1>
      <p class="text-lg sm:text-xl text-emerald-100/80 max-w-3xl mx-auto mb-10">
        {{ $t('hero.subtitle') }}
      </p>

      <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
        <NuxtLink
          v-if="latest"
          :to="localePath('/download' as any)"
          class="inline-flex items-center px-8 py-4 bg-gold-500 text-forest-950 rounded-xl font-bold text-lg shadow-lg shadow-gold-500/20 hover:bg-gold-400 transition"
        >
          {{ $t('hero.download') }}
          <span v-if="latest.version" class="ml-2 text-sm font-normal opacity-70">{{ latest.version }}</span>
        </NuxtLink>
        <a
          href="https://docs.oakvideoeditor.org"
          target="_blank"
          rel="noopener"
          class="inline-flex items-center px-8 py-4 border border-gold-500/40 text-gold-300 rounded-xl font-semibold text-lg hover:bg-gold-500/10 hover:border-gold-400 transition"
        >
          {{ $t('hero.docs') }}
        </a>
      </div>

      <p class="mt-4 text-sm text-emerald-100/50">
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
