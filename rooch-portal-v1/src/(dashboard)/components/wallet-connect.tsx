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

      toast.success('Connected to the wallet')
    } catch (error) {
      toast.error('Connection failed.')
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
              <div role="status">
                <svg
                  aria-hidden="true"
                  className="w-8 h-8 text-gray-200 animate-spin dark:text-gray-600 fill-gray-900"
                  viewBox="0 0 100 101"
                  fill="none"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
                    fill="currentColor"
                  />
                  <path
                    d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
                    fill="currentFill"
                  />
                </svg>
                <span className="sr-only">Loading...</span>
              </div>
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
                              className="ml-2 rounded-full border-teal-400 text-teal-400 hover:bg-teal-400/10"
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
