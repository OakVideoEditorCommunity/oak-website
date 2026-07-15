import type { DocsIndex } from '~/types'

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

  try {
    // Server-side fetch, so use the internal backend URL.
    const docs: DocsIndex = await $fetch(`${config.apiBaseUrl}/api/v1/docs`)
    for (const page of docs.zh) {
      urls.push({ loc: `${baseUrl}/docs/zh/${page.slug}`, changefreq: 'monthly', priority: 0.6 })
    }
    for (const page of docs.en) {
      urls.push({ loc: `${baseUrl}/docs/en/${page.slug}`, changefreq: 'monthly', priority: 0.6 })
      urls.push({ loc: `${baseUrl}/en/docs/en/${page.slug}`, changefreq: 'monthly', priority: 0.6 })
    }
  } catch (e) {
    console.error('Failed to fetch docs for sitemap', e)
  }

  return urls
})
