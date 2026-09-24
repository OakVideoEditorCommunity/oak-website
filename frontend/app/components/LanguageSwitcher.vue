<template>
  <label for="lang" class="text-sm text-emerald-100/60">Language:</label>
  <select
    :value="locale"
    class="bg-forest-800 border border-gold-500/30 text-emerald-100 text-sm rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-gold-400"
    id="lang"
    @change="switchLocale"
  >
    <option v-for="loc in locales" :key="loc.code" :value="loc.code">
      {{ loc.name }}
    </option>
  </select>
</template>

<script setup lang="ts">
const { locale, locales, setLocale } = useI18n()

// Do NOT bind the select via v-model on `locale`: writing the locale ref
// directly re-renders components before the target locale messages are
// lazily loaded, which makes $t fall back to raw i18n keys. `setLocale`
// awaits message loading and navigates to the localized route instead.
async function switchLocale(event: Event) {
  const target = event.target as HTMLSelectElement
  await setLocale(target.value)
}
</script>
