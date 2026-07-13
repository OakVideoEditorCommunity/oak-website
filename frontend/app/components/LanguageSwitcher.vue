<template>
  <select
    v-model="locale"
    class="bg-gray-100 border border-gray-300 text-gray-700 text-sm rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-emerald-500"
    @change="switchLocale"
  >
    <option v-for="loc in locales" :key="loc.code" :value="loc.code">
      {{ loc.name }}
    </option>
  </select>
</template>

<script setup lang="ts">
const { locale, locales, setLocale } = useI18n()

function switchLocale(event: Event) {
  const target = event.target as HTMLSelectElement
  setLocale(target.value)
}
</script>
