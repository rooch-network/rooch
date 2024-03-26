// import { ChainToggle } from './chain-toggle'
// import { NetworkToggle } from './network-toggle'
// import { Separator } from '@/components/ui/separator'
import { WalletConnect } from './wallet-connect'

export const NavbarItem = () => {
  return (
    <div className="flex items-center justify-end">
      {/* <ChainToggle /> */}
      {/* <NetworkToggle /> */}
      {/* <Separator orientation="vertical" className="h-6 hidden md:flex" /> */}
      <WalletConnect />
    </div>
  )
}
