import { createI18n } from 'vue-i18n'
import en from './locales/en'
import de from './locales/de'
import fr from './locales/fr'
import es from './locales/es'

export const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en, de, fr, es },
})
