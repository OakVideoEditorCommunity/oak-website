<template>
  <DocsPage v-if="versionKnown" :version="version" />
  <div v-else class="py-12 bg-gray-50 min-h-screen">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row gap-8">
      <article class="flex-1 bg-white rounded-2xl shadow p-8">
        <div class="py-20 text-center text-gray-500">{{ $t('docs.notFound') }}</div>
      </article>
    </div>
  </div>
</template>

<script setup lang="ts">
const route = useRoute()
const { versions } = useDocsVersions()

const version = computed(() => route.params.version as string)

// Until the versions list resolves (or if no versioned docs exist at all) we
// cannot tell whether the version is valid; render the page and let the doc
// fetch fall into its own not-found state instead of redirecting.
const versionKnown = computed(() => {
  if (versions.value.length === 0) return true
  return versions.value.includes(version.value)
})
</script>
