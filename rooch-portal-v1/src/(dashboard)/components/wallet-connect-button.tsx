import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'

import { Wallet } from 'lucide-react'

import { formatAddress } from '@/utils/format'
import { useConnectWallet, useWalletStore } from '@roochnetwork/rooch-sdk-kit'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ToastAction } from '@/components/ui/toast'
import { useToast } from '@/components/ui/use-toast'

export const WalletConnectButton = () => {
  const { toast } = useToast()
  const { mutateAsync: connectWallet } = useConnectWallet()
  const account = useWalletStore((state) => state.currentAccount)

  const handleWalletConnect = async () => {
    try {
      await connectWallet()
    } catch (error) {
      console.error('Wallet connection failed:', error)
    }
  }

  const handleWalletCardClick = () => {
    if (!account) {
      window.open(
        'https://chromewebstore.google.com/detail/unisat-wallet/ppbibelpcjmhbdihakflkdcoccbgbkpo',
        '_blank',
      )
    } else {
      toast({
        title: "You've already connected your wallet",
        description: (
          <span className="text-muted-foreground">Disconnect your wallet to connect a new one</span>
        ),
        action: <ToastAction altText="OK">💙 OK</ToastAction>,
      })
    }
  }

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button
          variant="default"
          size="sm"
          className="md:p-3 rounded-lg ml-2 h-auto md:h-9 p-2"
          onClick={handleWalletConnect}
        >
          <div className="flex items-center justify-center gap-x-2">
            <Wallet className="h-[1rem] w-[1rem] md:h-[1.2rem] md:w-[1.2rem] rotate-0 scale-100 transition-all" />
            <div className="flex items-center justify-center gap-x-1">
              {account === null ? 'Connect' : formatAddress(account?.getAddress())}
            </div>
          </div>
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle className="text-2xl text-center">Connect Wallet</DialogTitle>
        </DialogHeader>
        <Card
          onClick={handleWalletCardClick}
          className="bg-secondary cursor-pointer hover:border-primary/20 transition-all"
        >
          <CardHeader className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center justify-start">
                <img
                  src="/icon-unisat.svg"
                  alt="Unisat Icon"
                  className="h-[2rem] w-[2rem] rotate-0 scale-100 mr-2"
                />
                <div>
                  <CardTitle>Unisat Wallet</CardTitle>
                  <CardDescription>Connecting using Unisat Wallet</CardDescription>
                </div>
              </div>
              <img
                src="/icon-bitcoin.svg"
                alt="Unisat Icon"
                className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 mr-2"
              />
            </div>
          </CardHeader>
        </Card>
        <DialogFooter className="sm:justify-center">
          <span className="text-xs">
            Don't have a wallet?{' '}
            <span className="text-blue-400 font-medium hover:underline cursor-pointer hover:text-blue-300 transition-all">
              Download Rooch
            </span>
          </span>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
