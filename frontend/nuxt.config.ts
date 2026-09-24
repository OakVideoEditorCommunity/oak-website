// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  ssr: true,

  runtimeConfig: {
    apiBaseUrl: process.env.NUXT_API_BASE_URL || 'http://backend:8080',
    public: {
      // Empty means "same origin": the browser calls /api/... on this site's
      // own domain and the frontend's nginx forwards it to the backend.
      // This avoids CORS, mixed-content and Private Network Access issues.
      // NOTE: the site is fully static (nuxi generate), so public runtime
      // config is baked in at build time — set NUXT_PUBLIC_* before building.
      apiBaseUrl: process.env.NUXT_PUBLIC_API_BASE_URL || '',
      siteUrl: process.env.NUXT_PUBLIC_SITE_URL || 'http://localhost:3000',
      cdnDomain: process.env.NUXT_PUBLIC_CDN_DOMAIN || '',
    },
  },

  modules: [
    '@nuxtjs/tailwindcss',
    '@nuxtjs/sitemap',
    '@nuxtjs/i18n',
  ],

  i18n: {
    strategy: 'prefix_except_default',
    defaultLocale: 'en',
    locales: [
      { code: 'zh', name: '中文', file: 'zh.json' },
      { code: 'en', name: 'English', file: 'en.json' },
    ],
    langDir: 'locales/',
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: 'i18n_redirected',
      redirectOn: 'root',
      fallbackLocale: 'en',
    },
  },

  sitemap: {
    // Static pages are discovered from the prerendered routes; keep the
    // redirect stubs and the admin form out of the index.
    exclude: [
      '/admin',
      '/zh/admin',
      '/docs',
      '/docs/**',
      '/zh/docs',
      '/zh/docs/**',
    ],
  },

  // Feeds @nuxtjs/sitemap its canonical site URL (same value as
  // runtimeConfig.public.siteUrl; the module does not read that key itself).
  site: {
    url: process.env.NUXT_PUBLIC_SITE_URL || 'http://localhost:3000',
  },

  nitro: {
    prerender: {
      // The language switcher is a <select> (no crawlable <a>), and data is
      // fetched client-side, so enumerate every static route explicitly.
      // Dynamic routes (/releases/[version], versioned /docs/*) resolve
      // client-side through the static host's SPA fallback.
      routes: [
        '/',
        '/features',
        '/download',
        '/bug-report',
        '/admin',
        '/docs',
        '/zh',
        '/zh/features',
        '/zh/download',
        '/zh/bug-report',
        '/zh/admin',
        '/zh/docs',
      ],
    },
    cdnURL: process.env.NUXT_PUBLIC_CDN_DOMAIN || undefined,
  },

  app: {
    head: {
      charset: 'utf-8',
      viewport: 'width=device-width, initial-scale=1',
      meta: [
        { name: 'theme-color', content: '#081712' },
      ],
      link: [
        // Modern browsers pick the SVG logo; favicon.ico (16/32/48) is the fallback.
        { rel: 'icon', type: 'image/svg+xml', href: '/Oak_Icon.svg' },
        { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
      ],
    },
  },
})
