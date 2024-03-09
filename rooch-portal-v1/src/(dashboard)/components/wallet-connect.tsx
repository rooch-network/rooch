import { Wallet } from 'lucide-react'

import { formatAddress } from '@/utils/format'
import { useConnectWallet, useWalletStore } from '@roochnetwork/rooch-sdk-kit'

import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

export const WalletConnect = () => {
  const { mutateAsync: connectWallet } = useConnectWallet()
  const account = useWalletStore((state) => state.currentAccount)

  const handleWalletConnect = async () => {
    try {
      await connectWallet()
    } catch (error) {
      console.error('Wallet connection failed:', error)
    }
  }

  return (
    <>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="default" size="sm" className="flex ml-2" onClick={handleWalletConnect}>
              <div className="flex items-center justify-center gap-x-2">
                <Wallet className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all" />
                {account === null ? 'Connect' : formatAddress(account?.getAddress())}
              </div>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>Your wallet address</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </>
  )
}
