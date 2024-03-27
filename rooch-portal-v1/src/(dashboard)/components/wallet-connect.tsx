import { useState } from 'react'
import { Wallet } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

import toast from 'react-hot-toast'

import { formatAddress } from '@/utils/format'

import { useConnectWallet, useWalletStore, useWallets } from '@roochnetwork/rooch-sdk-kit'

interface WalletsListProps {
  name: string
  icon: string
  description: string
  types: string[]
  link: string
}

const walletsList: WalletsListProps[] = [
  {
    name: 'Unisat',
    icon: '/icon-unisat.svg',
    description: 'Unisat wallet',
    types: ['btc'],
    link: 'https://chromewebstore.google.com/detail/unisat-wallet/ppbibelpcjmhbdihakflkdcoccbgbkpo',
  },
  {
    name: 'MetaMask',
    icon: '/icon-metamask.svg',
    description: 'Metmask wallet',
    types: ['eth', 'bsc'],
    link: 'https://chromewebstore.google.com/detail/metamask/nkbihfbeogaeaoehlefnkodbefgpgknn',
  },
  {
    name: 'OKX',
    icon: '/icon-okx.svg',
    description: 'OKX wallet',
    types: ['eth', 'btc'],
    link: 'https://chromewebstore.google.com/detail/okx-wallet/mcohilncbfahbmgdjkbpemcciiolgcge',
  },
]

export const WalletConnect = () => {
  const wallets = useWallets()
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const { mutateAsync: connectWallet } = useConnectWallet()
  const account = useWalletStore((state) => state.currentAccount)

  // 1. Check installed wallets

  // 2. Get rooch account

  // ** Connect wallet
  const handleConnectWallet = () => {
    if (!account) {
      setIsDialogOpen(true)
    } else {
      navigator.clipboard
        .writeText(account.address)
        .then(() => {
          toast('Copied to clipboard!', {
            icon: '🌟',
          })
        })
        .catch((err) => {
          console.error('Failed to copy:', err)
        })
    }
  }

  // ** Connect specific wallet
  const handleConnectSpecificWallet = async (walletName: string) => {
    if (account) {
      toast("You've already connected to the wallet!", {
        icon: '✨',
      })
      return
    }

    // Find the matching wallet by name
    const walletToConnect = wallets.find(
      (wallet) => wallet.name?.toLowerCase() === walletName.toLowerCase(),
    )

    if (!walletToConnect) {
      toast.error(`Wallet '${walletName}' not found.`)
      return
    }

    console.log(walletToConnect)

    try {
      await connectWallet({ wallet: walletToConnect })

      setIsDialogOpen(false)
      toast.success(`Connected to the wallet ${walletName}`)
    } catch (error) {
      // Assuming the error is an object with a message property
      const errorMessage =
        error instanceof Error ? error.message : 'An error occurred while connecting to the wallet.'
      toast.error(errorMessage)
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
            {account === null ? 'Connect Wallet' : formatAddress(account.address)}
          </div>
        </div>
      </Button>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogTrigger asChild />
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle className="text-2xl text-center">Connect Wallet</DialogTitle>
          </DialogHeader>
          {walletsList.map((wallet) => (
            <Card
              key={wallet.name}
              onClick={() => handleConnectSpecificWallet(wallet.name)}
              className="bg-secondary cursor-pointer hover:border-primary/20 transition-all"
            >
              <CardHeader className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center justify-start">
                    <img
                      src={wallet.icon}
                      alt={wallet.description}
                      className="h-[2rem] w-[2rem] rotate-0 scale-100 mr-4 object-cover"
                    />
                    <div>
                      <CardTitle>{wallet.name} Wallet</CardTitle>
                      <CardDescription>Connecting using {wallet.name} Wallet</CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center justify-center gap-1">
                    {wallet.types.map((type) => (
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
