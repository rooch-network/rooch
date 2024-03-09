import { useTranslation } from 'react-i18next'
import { Globe2, Languages } from 'lucide-react'
import { Button } from '@/components/ui/button'

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation()

  const toggleLanguage = () => {
    const newLanguage = i18n.language === 'en' ? 'zh' : 'en'
    i18n.changeLanguage(newLanguage)
  }

  const languageIcon = () => {
    return i18n.language === 'zh' ? (
      <Languages className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
    ) : (
      <Globe2 className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
    )
  }

  return (
    <>
      <div className="w-full">
        <Button
          variant="ghost"
          size="icon"
          className="select-none w-full text-muted-foreground hover:text-muted-foreground justify-start px-2"
          onClick={toggleLanguage}
        >
          {languageIcon()}
          <span className="ml-2">English</span>
        </Button>
      </div>
    </>
  )
}
