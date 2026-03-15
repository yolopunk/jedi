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
          primary: '#0088aa',      // Sci-Fi Cyan (light)
          secondary: '#008844',    // Sci-Fi Green (light)
          accent: '#aa00aa',       // Sci-Fi Magenta (light)
          error: '#cc0000',
          info: '#0088aa',
          success: '#008844',
          warning: '#aa6600',
        }
      },
      dark: {
        dark: true,
        colors: {
          background: '#111113',   // Slightly lighter for better visibility
          surface: '#18181b',      // Zinc-900
          primary: '#00ffff',       // Sci-Fi Cyan
          secondary: '#00ff88',     // Sci-Fi Green
          accent: '#ff00ff',        // Sci-Fi Magenta
          error: '#ff4444',         // Sci-Fi Red
          info: '#00ffff',          // Sci-Fi Cyan
          success: '#00ff88',       // Sci-Fi Green
          warning: '#ffaa00',       // Sci-Fi Amber
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
