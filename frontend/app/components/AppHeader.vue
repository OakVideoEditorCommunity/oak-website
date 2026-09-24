<template>
  <header class="bg-forest-950/80 backdrop-blur border-b border-gold-500/15 sticky top-0 z-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      <NuxtLink :to="localePath('/' as any)" class="flex items-center gap-2 font-bold text-xl text-gold-400 hover:text-gold-300 transition">
        <img src="/Oak_Icon.svg" alt="Oak" class="w-8 h-8" />
        Oak Video Editor
      </NuxtLink>

      <nav class="hidden md:flex items-center gap-6 text-emerald-100/80">
        <NuxtLink :to="localePath('/' as any)" class="hover:text-gold-300 transition">
          {{ $t('nav.home') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/features' as any)" class="hover:text-gold-300 transition">
          {{ $t('nav.features') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/download' as any)" class="hover:text-gold-300 transition">
          {{ $t('nav.download') }}
        </NuxtLink>
        <a href="https://docs.oakvideoeditor.org" target="_blank" rel="noopener" class="hover:text-gold-300 transition">
          {{ $t('nav.docs') }}
        </a>
      </nav>

      <div class="flex items-center gap-3">
        <NuxtLink
          :to="localePath('/download' as any)"
          class="hidden sm:inline-flex items-center px-4 py-2 bg-gold-500 text-forest-950 rounded-lg font-semibold hover:bg-gold-400 transition"
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
