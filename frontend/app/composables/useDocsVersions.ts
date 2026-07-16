import type { DocsVersions } from '~/types'

/**
 * Build the doc-page path for a given docs version. The default (latest)
 * version lives at the unprefixed /docs/{lang}/{slug}; pinned versions get an
 * extra /docs/{version}/... segment.
 */
export function docsDocPath(
  version: string | null | undefined,
  latest: string,
  lang: string,
  slug: string,
): string {
  if (!version || version === latest) {
    return `/docs/${lang}/${slug}`
  }
  return `/docs/${version}/${lang}/${slug}`
}

export function useDocsVersions() {
  const { fetchApi } = useApi()

  // Shared key: every docs component on the page reuses one fetch.
  const { data } = useAsyncData<DocsVersions>('docs-versions', async () => {
    try {
      return await fetchApi<DocsVersions>('/api/v1/docs/versions')
    } catch {
      return { versions: [], latest: '' }
    }
  })

  const versions = computed(() => data.value?.versions ?? [])
  const latest = computed(() => data.value?.latest ?? '')

  function docPath(version: string | null | undefined, lang: string, slug: string) {
    return docsDocPath(version, latest.value, lang, slug)
  }

  return { versions, latest, docPath }
}
