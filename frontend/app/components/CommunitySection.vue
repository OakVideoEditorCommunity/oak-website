<template>
  <section class="py-16 border-t border-gold-500/10">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-12">
        <h2 class="text-3xl font-bold text-emerald-50 mb-4">{{ $t('community.title') }}</h2>
        <p class="text-emerald-100/60 max-w-2xl mx-auto">{{ $t('community.subtitle') }}</p>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
        <a
          v-for="channel in channels"
          :key="channel.key"
          :href="channel.href"
          target="_blank"
          rel="noopener"
          class="group bg-forest-900 border border-gold-500/15 rounded-2xl p-6 hover:border-gold-500/40 hover:-translate-y-0.5 transition"
        >
          <div class="w-11 h-11 rounded-xl bg-gold-500/10 border border-gold-500/30 text-gold-400 flex items-center justify-center mb-4 group-hover:text-gold-300 [&>svg]:w-6 [&>svg]:h-6" v-html="channel.icon" />
          <h3 class="font-semibold text-emerald-50 mb-1">{{ $t(`community.${channel.key}.name`) }}</h3>
          <p class="text-sm text-emerald-100/60 mb-2">{{ $t(`community.${channel.key}.desc`) }}</p>
          <p class="text-xs text-gold-400/80 break-all">{{ channel.address }}</p>
        </a>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
const icons = {
  // Envelope (hand-drawn, stroke style).
  email: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m3 7 9 6 9-6"/></svg>',
  // Brand logos: single-path official marks from simple-icons (fill style).
  facebook: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M9.101 23.691v-7.98H6.627v-3.667h2.474v-1.58c0-4.085 1.848-5.978 5.858-5.978.401 0 .955.042 1.468.103a8.68 8.68 0 0 1 1.141.195v3.325a8.623 8.623 0 0 0-.653-.036 26.805 26.805 0 0 0-.733-.009c-.707 0-1.259.096-1.675.309a1.686 1.686 0 0 0-.679.622c-.258.42-.374.995-.374 1.752v1.297h3.919l-.386 2.103-.287 1.564h-3.246v8.245C19.396 23.238 24 18.179 24 12.044c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.628 3.874 10.35 9.101 11.647Z"/></svg>',
  telegram: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>',
  github: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>',
}

const channels = [
  { key: 'email', href: 'mailto:dev@oakvideoeditor.org', address: 'dev@oakvideoeditor.org', icon: icons.email },
  { key: 'facebook', href: 'https://www.facebook.com/groups/oakvideoeditor/', address: 'facebook.com/groups/oakvideoeditor', icon: icons.facebook },
  { key: 'telegram', href: 'https://t.me/oakvideoeditor', address: 't.me/oakvideoeditor', icon: icons.telegram },
  { key: 'github', href: 'https://github.com/OakVideoEditorCommunity/oak', address: 'github.com/OakVideoEditorCommunity', icon: icons.github },
]
</script>
