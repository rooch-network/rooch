import { useState } from 'react'
import toast from 'react-hot-toast'
import { Wallet } from 'lucide-react'

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

import { formatAddress } from '@/utils/format'
import { capitalizeFirstLetter, cn } from '@/lib/utils'
import { walletsMaterialMap } from '../common/constant'

import {
  BaseWallet,
  SupportChain,
  useConnectWallet,
  useWallets,
  useWalletStore,
} from '@roochnetwork/rooch-sdk-kit'
import { LoadingSpinner } from './loading-spinner'

export const WalletConnect = () => {
  const [isLoading, setIsLoading] = useState(false)
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const { mutateAsync: connectWallet } = useConnectWallet()
  const account = useWalletStore((state) => state.currentAccount)
  const [currentWallet, setCurrentWallet] = useState<BaseWallet | null>(null)
  const wallets = useWallets().filter((wallet) => wallet.isSupportChain(SupportChain.BITCOIN))

  // ** Connect wallet
  const handleConnectWallet = () => {
    setIsDialogOpen(true)
  }

  // ** Connect specific wallet
  const handleConnectSpecificWallet = async (wallet: BaseWallet) => {
    try {
      setIsLoading(true)
      await connectWallet({ wallet: wallet })

      setCurrentWallet(wallet)
      setIsDialogOpen(false)

      toast.success(`${wallet?.name} wallet connected`)
    } catch (error) {
      toast.error('Connection failed')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <>
      <Button
        variant="secondary"
        size="default"
        className="md:p-3 rounded-lg ml-2 h-auto md:h-9 p-2"
        onClick={handleConnectWallet}
      >
        <div className="flex items-center justify-center gap-x-2 ">
          <Wallet className="h-[1rem] w-[1rem] md:h-[1.2rem] md:w-[1.2rem] rotate-0 scale-100 transition-all text-teal-600" />
          <div className="flex items-center justify-center gap-x-1 bg-gradient-to-r bg-clip-text font-black dark:from-teal-500 dark:via-purple-500 dark:to-orange-500 text-transparent from-teal-600 via-purple-600 to-orange-600">
            {account === null ? 'Connect Wallet' : formatAddress(account?.address)}
          </div>
        </div>
      </Button>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogTrigger asChild />
        <DialogContent
          className={cn(
            'sm:max-w-[425px]',
            isLoading ? 'border-gray-500 dark:border-gray-600' : '',
          )}
        >
          {isLoading && (
            <div className="absolute inset-0 flex items-center justify-center bg-gray-500 bg-opacity-50 z-10">
              <LoadingSpinner />
            </div>
          )}
          <DialogHeader>
            <DialogTitle className="text-2xl text-center">Connect Wallet</DialogTitle>
          </DialogHeader>
          {wallets.map((wallet) => (
            <Card
              key={wallet.name}
              onClick={() => handleConnectSpecificWallet(wallet)}
              className="bg-secondary cursor-pointer hover:border-primary/20 transition-all"
            >
              <CardHeader className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center justify-start">
                    <img
                      src={walletsMaterialMap.get(wallet.name!)!.icon}
                      alt={walletsMaterialMap.get(wallet.name!)!.description}
                      className="h-[2rem] w-[2rem] rotate-0 scale-100 mr-4 object-cover"
                    />
                    <div>
                      <CardTitle>
                        <span className="flex items-center justify-start">
                          <p>{capitalizeFirstLetter(wallet.name!)} Wallet</p>
                          {currentWallet?.name === wallet.name && (
                            <Badge
                              variant="outline"
                              className="ml-2 border-teal-400 text-teal-400 hover:bg-teal-400/10 py-0 px-0.5 rounded-md"
                            >
                              Current
                            </Badge>
                          )}
                        </span>
                      </CardTitle>
                      <CardDescription>Connecting using {wallet.name} Wallet</CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center justify-center gap-1">
                    {walletsMaterialMap.get(wallet.name!)!.types.map((type) => (
                      <img
                        key={type}
                        src={`/icon-${type}.svg`}
                        alt="Unisat Icon"
                        className="h-[1.1rem] w-[1.1rem] rotate-0 scale-100"
                      />
                    ))}
                  </div>
                </div>
              </CardHeader>
            </Card>
          ))}
        </DialogContent>
      </Dialog>
    </>
  )
}
