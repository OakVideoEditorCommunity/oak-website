// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  ssr: true,

  runtimeConfig: {
    apiBaseUrl: process.env.NUXT_API_BASE_URL || 'http://backend:8080',
    public: {
      apiBaseUrl: process.env.NUXT_PUBLIC_API_BASE_URL || 'http://localhost:8080',
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
    sources: [
      '/api/__sitemap__/urls',
    ],
  },

  nitro: {
    routeRules: {
      '/': { prerender: true },
      '/download': { prerender: true },
      '/docs/**': { prerender: false, isr: 3600 },
    },
    cdnURL: process.env.NUXT_PUBLIC_CDN_DOMAIN || undefined,
  },

  app: {
    head: {
      charset: 'utf-8',
      viewport: 'width=device-width, initial-scale=1',
      link: [
        // Modern browsers pick the SVG logo; favicon.ico (16/32/48) is the fallback.
        { rel: 'icon', type: 'image/svg+xml', href: '/Oak_Icon.svg' },
        { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
      ],
    },
  },
})
