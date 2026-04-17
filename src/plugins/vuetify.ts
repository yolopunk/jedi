import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg'
import { zhHans } from 'vuetify/locale'

// We are now using CSS variables for the theme source of truth.
// Vuetify requires some colors to be defined for internal calculations,
// but our CSS overrides will take precedence for visual rendering.
// To ensure maximum compatibility, we map these to our variable names.

export const vuetify = createVuetify({
  components,
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: {
      mdi,
    },
  },
  theme: {
    defaultTheme: 'dark', // Default to dark for the Jedi vibe
    themes: {
      light: {
        dark: false,
        colors: {
          background: '#f5e6d3', // Warm sand
          surface: '#efe0cc', // Light sand
          primary: '#cd7f32', // Copper
          secondary: '#6b4423', // Sandstone brown
          accent: '#daa520', // Dark gold
          error: '#b22222', // Firebrick red
          info: '#b8860b', // Dark goldenrod
          success: '#daa520', // Dark gold
          warning: '#cd853f', // Peru
        },
      },
      dark: {
        dark: true,
        colors: {
          background: '#111113', // Slightly lighter for better visibility
          surface: '#18181b', // Zinc-900
          primary: '#00ffff', // Sci-Fi Cyan
          secondary: '#00ff88', // Sci-Fi Green
          accent: '#ff00ff', // Sci-Fi Magenta
          error: '#ff4444', // Sci-Fi Red
          info: '#00ffff', // Sci-Fi Cyan
          success: '#00ff88', // Sci-Fi Green
          warning: '#ffaa00', // Sci-Fi Amber
        },
      },
    },
  },
  locale: {
    locale: 'zhHans',
    fallback: 'en',
    messages: { zhHans },
  },
})
