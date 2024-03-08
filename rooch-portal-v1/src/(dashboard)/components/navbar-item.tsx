import { ChainToggle } from './chain-toggle'
import { WalletConnect } from './wallet-connect'
import { NetworkToggle } from './network-toggle'
import { Separator } from '@/components/ui/separator'
import { ModeToggle } from '@/components/mode-toggle'
import { LanguageSwitcher } from '@/components/language-switcher'

export const NavbarItem = () => {
  return (
    <div className="flex items-center justify-end md:gap-x-2 gap-x-0">
      <LanguageSwitcher />
      <ModeToggle />
      <ChainToggle />
      <NetworkToggle />
      <Separator orientation="vertical" className="h-6 md:flex hidden" />
      <WalletConnect />
    </div>
  )
}
