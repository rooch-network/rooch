import { LanguageSwitcher } from '@/components/language-switcher'
import { ModeToggle } from '@/components/mode-toggle'
import { WalletConnect } from './wallet-connect'
import { Separator } from '@/components/ui/separator'

export const NavbarItem = () => {
  return (
    <div className="flex items-center justify-end md:gap-x-2 gap-x-0">
      <LanguageSwitcher />
      <ModeToggle />
      <Separator orientation="vertical" className="h-6 md:flex hidden" />
      <WalletConnect />
    </div>
  )
}
