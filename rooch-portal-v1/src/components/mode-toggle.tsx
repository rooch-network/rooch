import { Moon, Settings, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/components/theme-provider'
import { useTranslation } from 'react-i18next'

export const ModeToggle = () => {
  const { t } = useTranslation()
  const { theme, setTheme } = useTheme()

  const ThemeIcon = () => {
    switch (theme) {
      case 'light':
        return <Sun className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
      case 'dark':
        return <Moon className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
      default:
        return <Settings className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
    }
  }

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
  }

  return (
    <>
      <div className="flex w-full">
        <Button
          variant="ghost"
          size="sm"
          className="select-none text-muted-foreground hover:text-muted-foreground justify-start px-2 w-full"
          onClick={toggleTheme}
        >
          <ThemeIcon />
          <span className="ml-2">{t(`Theme.${theme}`)}</span>
        </Button>
      </div>
    </>
  )
}
