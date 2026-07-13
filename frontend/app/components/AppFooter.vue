<template>
  <footer class="bg-gray-900 text-gray-300 py-10">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 grid grid-cols-1 md:grid-cols-3 gap-8">
      <div>
        <h3 class="text-white font-bold text-lg mb-3">Oak Video Editor</h3>
        <p class="text-sm">{{ $t('footer.tagline') }}</p>
      </div>
      <div>
        <h4 class="text-white font-semibold mb-3">{{ $t('footer.links') }}</h4>
        <ul class="space-y-2 text-sm">
          <li><NuxtLink :to="localePath('/' as any)" class="hover:text-white">{{ $t('nav.home') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/download' as any)" class="hover:text-white">{{ $t('nav.download') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/docs' as any)" class="hover:text-white">{{ $t('nav.docs') }}</NuxtLink></li>
        </ul>
      </div>
      <div>
        <h4 class="text-white font-semibold mb-3">{{ $t('footer.community') }}</h4>
        <ul class="space-y-2 text-sm">
          <li><a href="https://github.com/OakVideoEditorCommunity/oak" target="_blank" rel="noopener" class="hover:text-white">GitHub</a></li>
        </ul>
      </div>
    </div>
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8 pt-8 border-t border-gray-800 text-sm text-center">
      © {{ new Date().getFullYear() }} Oak Video Editor Community
    </div>
  </footer>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
