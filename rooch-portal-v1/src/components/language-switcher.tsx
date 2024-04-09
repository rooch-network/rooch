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
      <Languages className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100" />
    ) : (
      <Globe2 className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100" />
    )
  }

  const languageLabel = () => {
    return i18n.language === 'zh' ? '简体中文' : 'English'
  }

  return (
    <>
      <div className="w-full transition-all">
        <Button
          variant="ghost"
          size="sm"
          className="select-none w-full text-muted-foreground hover:text-muted-foreground justify-start px-2 hover:bg-zinc-300/20 dark:hover:bg-zinc-800/50 hover:text-zinc-600 dark:hover:text-white"
          onClick={toggleLanguage}
        >
          {languageIcon()}
          <span className="ml-2">{languageLabel()}</span>
        </Button>
      </div>
    </>
  )
}
