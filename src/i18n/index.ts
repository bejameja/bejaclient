import { createI18n } from 'vue-i18n'
import en from './locales/en'
import de from './locales/de'
import fr from './locales/fr'
import es from './locales/es'
import ru from './locales/ru'
import zh from './locales/zh'
import hi from './locales/hi'

export const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en, de, fr, es, ru, zh, hi },
})
