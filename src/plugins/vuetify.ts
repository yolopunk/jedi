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
  // Colors mirror the semantic tokens in src/assets/theme.css (single source of truth).
  // CSS variables drive visual rendering; these keep Vuetify's internal calculations aligned.
  theme: {
    defaultTheme: 'dark', // Default to dark for the Jedi vibe
    themes: {
      light: {
        dark: false,
        colors: {
          background: '#ffffff', // Int UI Grey 14
          surface: '#f7f8fa', // Int UI Grey 13
          primary: '#3574f0', // Int UI Blue 4
          secondary: '#6c707e', // Int UI Grey 6 (neutral, so text-secondary reads muted)
          accent: '#834df0', // Int UI Purple
          error: '#db3b4b', // Int UI Red 3
          info: '#3574f0', // Int UI Blue 4
          success: '#208a3c', // Int UI Green 3
          warning: '#a46704', // Int UI Yellow (dark)
        },
      },
      dark: {
        dark: true,
        colors: {
          background: '#111113', // Zinc near-black
          surface: '#18181b', // Zinc-900
          primary: '#38bdf8', // Unified sci-fi cyan (was #00ffff)
          secondary: '#c084fc', // Purple-400
          accent: '#c084fc', // Purple-400
          error: '#f87171', // Red-400
          info: '#38bdf8', // Cyan
          success: '#4ade80', // Green-400
          warning: '#fbbf24', // Amber-400
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
