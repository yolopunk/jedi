import 'vuetify/styles'
import '../styles.css'
import { createVuetify } from 'vuetify'
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg'
import * as components from 'vuetify/components'
import { zhHans } from 'vuetify/locale'

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
  },
  locale: {
    locale: 'zhHans',
    fallback: 'en',
    messages: { zhHans }
  }
})