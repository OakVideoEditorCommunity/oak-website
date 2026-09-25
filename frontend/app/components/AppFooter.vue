<template>
  <footer class="bg-black/40 border-t border-gold-500/10 text-emerald-100/70 py-10">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 grid grid-cols-1 md:grid-cols-3 gap-8">
      <div>
        <h3 class="text-gold-400 font-bold text-lg mb-3">Oak Video Editor</h3>
        <p class="text-sm">{{ $t('footer.tagline') }}</p>
      </div>
      <div>
        <h4 class="text-emerald-50 font-semibold mb-3">{{ $t('footer.links') }}</h4>
        <ul class="space-y-2 text-sm">
          <li><NuxtLink :to="localePath('/' as any)" class="hover:text-gold-300 transition">{{ $t('nav.home') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/features' as any)" class="hover:text-gold-300 transition">{{ $t('nav.features') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/download' as any)" class="hover:text-gold-300 transition">{{ $t('nav.download') }}</NuxtLink></li>
          <li><a href="https://docs.oakvideoeditor.org" target="_blank" rel="noopener" class="hover:text-gold-300 transition">{{ $t('nav.docs') }}</a></li>
          <li><NuxtLink :to="localePath('/bug-report' as any)" class="hover:text-gold-300 transition">{{ $t('footer.bugReport') }}</NuxtLink></li>
        </ul>
      </div>
      <div>
        <h4 class="text-emerald-50 font-semibold mb-3">{{ $t('footer.community') }}</h4>
        <ul class="space-y-2 text-sm">
          <li><a href="mailto:dev@oakvideoeditor.org" class="hover:text-gold-300 transition">{{ $t('community.email.name') }}</a></li>
          <li><a href="https://www.facebook.com/groups/oakvideoeditor/" target="_blank" rel="noopener" class="hover:text-gold-300 transition">Facebook</a></li>
          <li><a href="https://t.me/oakvideoeditor" target="_blank" rel="noopener" class="hover:text-gold-300 transition">Telegram</a></li>
          <li><a href="https://github.com/OakVideoEditorCommunity/oak" target="_blank" rel="noopener" class="hover:text-gold-300 transition">GitHub</a></li>
        </ul>
      </div>
    </div>
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8 pt-8 border-t border-gold-500/10 text-sm text-center text-emerald-100/50">
      © {{ new Date().getFullYear() }} Oak Video Editor Community
    </div>
  </footer>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
