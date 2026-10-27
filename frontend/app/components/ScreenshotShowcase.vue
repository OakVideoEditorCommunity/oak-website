<template>
  <section class="relative py-16 sm:py-20 overflow-hidden" aria-labelledby="screenshot-title">
    <div class="absolute inset-x-0 top-1/2 -translate-y-1/2 h-96 bg-gold-500/5 blur-3xl pointer-events-none" aria-hidden="true" />

    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-10">
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-gold-400/80 mb-3">
          {{ $t('screenshot.eyebrow') }}
        </p>
        <h2 id="screenshot-title" class="text-3xl sm:text-4xl font-bold text-emerald-50 mb-4">
          {{ $t('screenshot.title') }}
        </h2>
        <p class="text-emerald-100/60 max-w-2xl mx-auto">
          {{ $t('screenshot.subtitle') }}
        </p>
      </div>

      <figure class="relative mx-auto max-w-6xl">
        <div class="absolute -inset-px rounded-2xl bg-gradient-to-r from-gold-500/40 via-gold-300/20 to-gold-500/40 blur-sm" aria-hidden="true" />
        <div class="relative rounded-2xl border border-gold-500/30 bg-forest-900/90 p-2 sm:p-3 shadow-2xl shadow-black/60">
          <div class="overflow-hidden rounded-xl border border-white/10 bg-black">
            <img
              src="/images/oak-editor-screenshot.webp"
              :alt="$t('screenshot.alt')"
              width="2680"
              height="1564"
              loading="lazy"
              decoding="async"
              class="block w-full h-auto"
            />
          </div>
          <figcaption class="mt-3 flex items-center justify-center gap-2 text-xs sm:text-sm text-emerald-100/50">
            <span class="inline-block w-1.5 h-1.5 rounded-full bg-gold-500/80" aria-hidden="true" />
            {{ $t('screenshot.caption') }}
          </figcaption>
        </div>
      </figure>
    </div>
  </section>
</template>
