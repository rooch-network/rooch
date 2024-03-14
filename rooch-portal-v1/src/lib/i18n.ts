import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import zhTranslation from '../locales/zh.json'
import enTranslation from '../locales/en.json'

i18n.use(initReactI18next).init({
  debug: false,
  resources: {
    en: { translation: enTranslation },
    zh: { translation: zhTranslation },
  },
  lng: 'en',
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
})

export default i18n
