import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg'
import * as components from 'vuetify/components'
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
      mdi
    }
  },
  theme: {
    defaultTheme: 'dark', // Default to dark for the Jedi vibe
    themes: {
      light: {
        dark: false,
        colors: {
          background: '#ffffff',
          surface: '#f4f4f5',
          primary: '#09090b',
          secondary: '#71717a',
          error: '#ef4444',
          info: '#3b82f6',
          success: '#10b981',
          warning: '#f59e0b',
        }
      },
      dark: {
        dark: true,
        colors: {
          background: '#09090b', // Zinc-950
          surface: '#18181b',    // Zinc-900
          primary: '#60a5fa',    // Blue-400
          secondary: '#a1a1aa',  // Zinc-400
          error: '#f87171',      // Red-400
          info: '#60a5fa',       // Blue-400
          success: '#4ade80',    // Green-400
          warning: '#fbbf24',    // Amber-400
        }
      }
    }
  },
  locale: {
    locale: 'zhHans',
    fallback: 'en',
    messages: { zhHans }
  }
})
