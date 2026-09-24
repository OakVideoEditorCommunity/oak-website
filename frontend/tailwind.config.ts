import type { Config } from 'tailwindcss'

export default <Partial<Config>>{
  content: [],
  theme: {
    extend: {
      colors: {
        // Deep green surfaces (near-black page background -> lifted cards).
        forest: {
          300: '#7dbc9c',
          400: '#4c9a77',
          500: '#2f7a5b',
          600: '#256049',
          700: '#1d4a39',
          800: '#14352a',
          900: '#0d231b',
          950: '#081712',
        },
        // Metallic gold accents (primary CTA, highlights, borders).
        gold: {
          300: '#f0d68a',
          400: '#e3c566',
          500: '#d4af37',
          600: '#b8941f',
          700: '#8a6d14',
        },
        emerald: {
          50: '#ecfdf5',
          100: '#d1fae5',
          200: '#a7f3d0',
          300: '#6ee7b7',
          400: '#34d399',
          500: '#10b981',
          600: '#059669',
          700: '#047857',
          800: '#065f46',
          900: '#064e3b',
        },
      },
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
