import { createI18n } from 'vue-i18n'
import zh from './locales/zh'
import en from './locales/en'

// Create i18n instance with options
const i18n = createI18n({
  legacy: false, // Use Composition API
  locale: 'zh', // set locale
  fallbackLocale: 'en', // set fallback locale
  messages: {
    zh,
    en
  }
})

export default i18n
