import { useTranslation } from 'react-i18next'
import { Globe2, Languages } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation()

  const switchLanguage = (language: string) => {
    i18n.changeLanguage(language)
  }

  const languageIcon = () => {
    return i18n.language === 'zh' ? (
      <Languages className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
    ) : (
      <Globe2 className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
    )
  }

  const getLanguageName = () => {
    return i18n.language === 'zh' ? '简体中文' : 'English'
  }

  const dropdownMenuItems = () => (
    <DropdownMenuContent align="end">
      <DropdownMenuItem onClick={() => switchLanguage('en')}>
        <div className="flex items-center justify-start gap-x-2">
          <Globe2 className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
          English
        </div>
      </DropdownMenuItem>
      <DropdownMenuItem onClick={() => switchLanguage('zh')}>
        <div className="flex items-center justify-start gap-x-2">
          <Languages className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
          简体中文
        </div>
      </DropdownMenuItem>
    </DropdownMenuContent>
  )

  return (
    <>
      {/* Mobile version */}
      <div className="md:hidden">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="select-none">
              {languageIcon()}
              <span className="sr-only">Change language</span>
            </Button>
          </DropdownMenuTrigger>
          {dropdownMenuItems()}
        </DropdownMenu>
      </div>

      {/* Desktop version */}
      <div className="hidden md:flex">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => switchLanguage(i18n.language === 'en' ? 'zh' : 'en')}
              className="flex items-center justify-center select-none"
            >
              {languageIcon()}
              <span className="ml-2">{getLanguageName()}</span>
            </Button>
          </DropdownMenuTrigger>
          {dropdownMenuItems()}
        </DropdownMenu>
      </div>
    </>
  )
}
