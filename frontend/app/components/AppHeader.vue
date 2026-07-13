<template>
  <header class="bg-white shadow-sm sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      <NuxtLink :to="localePath('/' as any)" class="flex items-center gap-2 font-bold text-xl text-emerald-700">
        <img src="/favicon.ico" alt="Oak" class="w-8 h-8" />
        Oak Video Editor
      </NuxtLink>

      <nav class="hidden md:flex items-center gap-6">
        <NuxtLink :to="localePath('/' as any)" class="hover:text-emerald-600 transition">
          {{ $t('nav.home') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/download' as any)" class="hover:text-emerald-600 transition">
          {{ $t('nav.download') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/docs' as any)" class="hover:text-emerald-600 transition">
          {{ $t('nav.docs') }}
        </NuxtLink>
      </nav>

      <div class="flex items-center gap-3">
        <NuxtLink
          :to="localePath('/download' as any)"
          class="hidden sm:inline-flex items-center px-4 py-2 bg-emerald-600 text-white rounded-lg font-medium hover:bg-emerald-700 transition"
        >
          {{ $t('download.now') }}
        </NuxtLink>
        <LanguageSwitcher />
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
