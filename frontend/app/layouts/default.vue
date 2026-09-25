<template>
  <div class="min-h-screen flex flex-col bg-forest-950 text-emerald-50">
    <AppHeader />
    <div class="bg-gold-500/10 border-b border-gold-500/25 text-center text-sm px-4 py-2">
      <span class="text-gold-300/90">{{ $t('notice.alpha') }}</span>
      <NuxtLink :to="localePath('/bug-report' as any)" class="ml-2 text-gold-400 font-medium hover:text-gold-300 underline underline-offset-2 transition">
        {{ $t('notice.reportLink') }}
      </NuxtLink>
    </div>
    <main class="flex-1">
      <slot />
    </main>
    <AppFooter />
  </div>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
