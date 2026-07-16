import type { DocsIndex, DocsVersions } from '~/types'

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig()
  const baseUrl = config.public.siteUrl.replace(/\/$/, '')

  const urls = [
    { loc: `${baseUrl}/`, changefreq: 'weekly', priority: 1.0 },
    { loc: `${baseUrl}/download`, changefreq: 'weekly', priority: 0.9 },
    { loc: `${baseUrl}/docs`, changefreq: 'weekly', priority: 0.8 },
    { loc: `${baseUrl}/en/`, changefreq: 'weekly', priority: 1.0 },
    { loc: `${baseUrl}/en/download`, changefreq: 'weekly', priority: 0.9 },
    { loc: `${baseUrl}/en/docs`, changefreq: 'weekly', priority: 0.8 },
  ]

  // The latest version keeps the canonical unprefixed URLs (with the /en
  // locale variants); older versions get /docs/{version}/{lang}/{slug}.
  const addVersioned = (docs: DocsIndex, version: string, isLatest: boolean) => {
    for (const page of docs.zh) {
      const path = isLatest ? `/docs/zh/${page.slug}` : `/docs/${version}/zh/${page.slug}`
      urls.push({ loc: `${baseUrl}${path}`, changefreq: 'monthly', priority: 0.6 })
    }
    for (const page of docs.en) {
      const path = isLatest ? `/docs/en/${page.slug}` : `/docs/${version}/en/${page.slug}`
      urls.push({ loc: `${baseUrl}${path}`, changefreq: 'monthly', priority: 0.6 })
      if (isLatest) {
        urls.push({ loc: `${baseUrl}/en/docs/en/${page.slug}`, changefreq: 'monthly', priority: 0.6 })
      }
    }
  }

  try {
    // Server-side fetch, so use the internal backend URL.
    const docsVersions = await $fetch<DocsVersions>(`${config.apiBaseUrl}/api/v1/docs/versions`).catch(() => null)

    if (docsVersions && docsVersions.versions.length > 0) {
      const tocs = await Promise.all(
        docsVersions.versions.map((version) =>
          $fetch<DocsIndex>(`${config.apiBaseUrl}/api/v1/docs?version=${encodeURIComponent(version)}`).catch(() => null)
        )
      )
      docsVersions.versions.forEach((version, i) => {
        const toc = tocs[i]
        if (toc) {
          addVersioned(toc, version, version === docsVersions.latest)
        }
      })
    } else {
      // No versioned docs (or an older backend): fall back to the unversioned
      // listing, which always serves the default version.
      const docs: DocsIndex = await $fetch(`${config.apiBaseUrl}/api/v1/docs`)
      addVersioned(docs, '', true)
    }
  } catch (e) {
    console.error('Failed to fetch docs for sitemap', e)
  }

  return urls
})
