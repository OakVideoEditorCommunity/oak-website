<template>
  <div>
    <section class="relative overflow-hidden py-20 bg-gradient-to-br from-forest-900 via-forest-950 to-black">
      <div class="absolute -top-32 left-1/2 -translate-x-1/2 w-[36rem] h-[36rem] rounded-full bg-gold-500/10 blur-3xl pointer-events-none" />
      <div class="relative max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <h1 class="text-4xl sm:text-5xl font-extrabold tracking-tight mb-4 text-transparent bg-clip-text bg-gradient-to-r from-gold-300 via-gold-400 to-gold-500">
          {{ $t('featuresPage.title') }}
        </h1>
        <p class="text-lg text-emerald-100/70 max-w-2xl mx-auto">
          {{ $t('featuresPage.subtitle') }}
        </p>
      </div>
    </section>

    <section class="py-12">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 space-y-8">
        <article
          v-for="feature in features"
          :key="feature.key"
          :id="feature.key"
          class="bg-forest-900 border border-gold-500/15 rounded-2xl shadow shadow-black/30 p-8 hover:border-gold-500/40 transition"
        >
          <div class="flex flex-col sm:flex-row items-start gap-5">
            <div
              class="shrink-0 w-12 h-12 rounded-xl bg-gold-500/10 border border-gold-500/30 text-gold-400 flex items-center justify-center [&>svg]:w-6 [&>svg]:h-6"
              v-html="feature.icon"
            />
            <div class="flex-1">
              <h2 class="text-2xl font-bold text-emerald-50">{{ $t(`featuresPage.${feature.key}.title`) }}</h2>
              <p class="text-gold-400/90 text-sm font-medium mb-3">{{ $t(`featuresPage.${feature.key}.tagline`) }}</p>
              <p class="text-emerald-100/70 leading-relaxed mb-4">{{ $t(`featuresPage.${feature.key}.desc`) }}</p>
              <ul class="space-y-1.5">
                <li
                  v-for="point in tm(`featuresPage.${feature.key}.points`)"
                  :key="rt(point)"
                  class="flex items-start gap-2 text-sm text-emerald-100/60"
                >
                  <span class="text-gold-500 mt-0.5">◆</span>
                  <span>{{ rt(point) }}</span>
                </li>
              </ul>
              <p v-if="hasNote(feature.key)" class="mt-4 text-xs text-emerald-100/40">
                {{ $t(`featuresPage.${feature.key}.note`) }}
              </p>
            </div>
          </div>
        </article>
      </div>
    </section>

    <section class="py-16 bg-gradient-to-br from-forest-900 to-forest-950 border-t border-gold-500/10">
      <div class="max-w-4xl mx-auto px-4 text-center">
        <h2 class="text-3xl font-bold text-emerald-50 mb-4">{{ $t('cta.title') }}</h2>
        <p class="text-emerald-100/60 mb-8">{{ $t('cta.subtitle') }}</p>
        <NuxtLink
          :to="localePath('/download' as any)"
          class="inline-flex items-center px-8 py-4 bg-gold-500 text-forest-950 rounded-xl font-bold text-lg shadow-lg shadow-gold-500/20 hover:bg-gold-400 transition"
        >
          {{ $t('cta.button') }}
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
const { t, tm, rt, te } = useI18n()
const localePath = useLocalePath()

const icons: Record<string, string> = {
  // Sliders — color grading with OCIO.
  ocio: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M4 7h16M4 12h16M4 17h16"/><circle cx="9" cy="7" r="2" fill="currentColor" stroke="none"/><circle cx="15" cy="12" r="2" fill="currentColor" stroke="none"/><circle cx="7" cy="17" r="2" fill="currentColor" stroke="none"/></svg>',
  // Grid + plus — loadable OpenFX plug-ins.
  openfx: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><path d="M16.5 13.5v6M13.5 16.5h6" stroke-linecap="round"/></svg>',
  // Left-right arrows — timeline interchange.
  otio: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 8h13l-3-3M20 16H7l3 3"/></svg>',
  // Palette — color management.
  colorMgmt: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3a9 9 0 1 0 0 18h1a2.5 2.5 0 0 0 0-5h-1a2 2 0 0 1 0-4h5a4 4 0 0 0 4-4c0-2.8-4-5-9-5Z" stroke-linejoin="round"/><circle cx="7.5" cy="10.5" r="1" fill="currentColor"/><circle cx="12" cy="7.5" r="1" fill="currentColor"/><circle cx="16.5" cy="10.5" r="1" fill="currentColor"/></svg>',
  // Monitor — 10-bit display output.
  tenBit: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="4" width="18" height="12" rx="2"/><path d="M9 20h6M12 16v4" stroke-linecap="round"/></svg>',
}

const features = [
  { key: 'ocio', icon: icons.ocio },
  { key: 'openfx', icon: icons.openfx },
  { key: 'otio', icon: icons.otio },
  { key: 'colorMgmt', icon: icons.colorMgmt },
  { key: 'tenBit', icon: icons.tenBit },
]

function hasNote(key: string) {
  return te(`featuresPage.${key}.note`)
}

useOakSeo({
  title: t('seo.featuresTitle'),
  description: t('seo.featuresDescription'),
})
</script>
