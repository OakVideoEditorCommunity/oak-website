interface SeoOptions {
  title?: string
  description?: string
  image?: string
  url?: string
  canonicalPath?: string
  type?: string
  jsonLd?: Record<string, any>
}

export function useOakSeo(options: SeoOptions = {}) {
  const { t } = useI18n()
  const config = useRuntimeConfig()
  const route = useRoute()

  const siteName = 'Oak Video Editor'
  const defaultDescription = t('seo.defaultDescription')
  const title = options.title ? `${options.title} - ${siteName}` : siteName
  const description = options.description || defaultDescription
  const url = options.url || `${config.public.siteUrl}${route.path}`
  // Pinned-version pages keep og:url on the real URL but canonicalize to the
  // default (latest) docs URL.
  const canonicalUrl = options.canonicalPath
    ? `${config.public.siteUrl}${options.canonicalPath}`
    : url
  const image = options.image || config.public.cdnDomain || ''

  const meta: any[] = [
    { name: 'description', content: description },
    { property: 'og:title', content: title },
    { property: 'og:description', content: description },
    { property: 'og:url', content: url },
    { property: 'og:type', content: options.type || 'website' },
    { property: 'og:site_name', content: siteName },
    { name: 'twitter:card', content: 'summary_large_image' },
    { name: 'twitter:title', content: title },
    { name: 'twitter:description', content: description },
  ]

  if (image) {
    meta.push({ property: 'og:image', content: image })
    meta.push({ name: 'twitter:image', content: image })
  }

  useHead({
    title,
    meta,
    link: [
      { rel: 'canonical', href: canonicalUrl },
    ],
  })

  if (options.jsonLd) {
    useHead({
      script: [
        {
          type: 'application/ld+json',
          innerHTML: JSON.stringify(options.jsonLd),
        },
      ],
    })
  }
}
