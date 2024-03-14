import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { WalletConnectButton } from './wallet-connect-button'

export const WalletConnect = () => {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger className="hidden md:flex" asChild>
          <WalletConnectButton />
        </TooltipTrigger>
        <TooltipContent>
          <p>Connect to your wallet</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
