import { Moon, Settings, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
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

  const getThemeName = () => {
    switch (theme) {
      case 'light':
        return t('Theme.light')
      case 'dark':
        return t('Theme.dark')
      default:
        return t('Theme.system')
    }
  }

  const dropdownMenu = () => {
    return (
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme('light')}>
          <div className="flex items-center justify-start gap-x-2">
            <Sun className="h-[1rem] w-[1rem] rotate-0 transition-all" />
            {t('Theme.light')}
          </div>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('dark')}>
          <div className="flex items-center justify-start gap-x-2">
            <Moon className="h-[1rem] w-[1rem] rotate-0 transition-all " />
            {t('Theme.dark')}
          </div>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('system')}>
          <div className="flex items-center justify-start gap-x-2">
            <Settings className="h-[1rem] w-[1rem] rotate-0 transition-all" />
            {t('Theme.system')}
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    )
  }

  return (
    <>
      {/* mobile */}
      <div className="md:hidden">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="select-none">
              <ThemeIcon />
              <span className="sr-only">Toggle theme</span>
            </Button>
          </DropdownMenuTrigger>
          {dropdownMenu()}
        </DropdownMenu>
      </div>

      {/* desktop */}
      <div className="hidden md:flex">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm" className="select-none">
              <ThemeIcon />
              <span className="ml-2">{getThemeName()}</span>
            </Button>
          </DropdownMenuTrigger>
          {dropdownMenu()}
        </DropdownMenu>
      </div>
    </>
  )
}
