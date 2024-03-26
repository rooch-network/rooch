import { useEffect, useState } from 'react'
import { Wallet } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  // DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

import toast from 'react-hot-toast'

import { formatAddress } from '@/utils/format'

import { useConnectWallet, useWalletStore } from '@roochnetwork/rooch-sdk-kit'
import { BaseWallet } from '@roochnetwork/rooch-sdk-kit/src/types/wellet/baseWallet'
import { getInstalledWallets } from '@roochnetwork/rooch-sdk-kit/src/utils/walletUtils'
import { SupportChain } from '@roochnetwork/rooch-sdk-kit/src/feature'

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
  const [wallets, setWallets] = useState<BaseWallet[]>() // Get installed wallets
  const [chain] = useState(SupportChain.BITCOIN) // 目前默认只有 BITCOIN
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const { mutateAsync: connectWallet } = useConnectWallet()
  const account = useWalletStore((state) => state.currentAccount)

  useEffect(() => {
    getInstalledWallets(chain).then((v) => setWallets(v))
  }, [chain])

  // using `wallets` to pass the compiling
  console.log(wallets)

  // - TEST
  // const wallet = useCurrentWallet()
  // const walletStore = useWalletStore((state) => state.accounts)

  // console.log('Wallet', wallet)
  // console.log('Wallet Store', walletStore)3
  // - TEST

  // `createSessionAccount()` in account.ts

  // ** TODO: isWalletInstalled

  // ** TODO: Get rooch account
  // account.getRoochAddress()

  // ** Connect wallet
  const handleConnectWallet = () => {
    if (!account) {
      setIsDialogOpen(true)
    } else {
      navigator.clipboard
        .writeText(account.getAddress())
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
    if (!account) {
      try {
        switch (walletName) {
          case 'Unisat':
            // 1. Handle connect to Unisat
            await connectWallet()
            break
          case 'MetaMask':
            // 2. Handle connect to MetaMask
            break
          case 'OKX':
            // 3. Handle connect to OKX
            break
          default:
            await connectWallet()
        }

        setIsDialogOpen(false)
        toast.success('Connected to the wallet')
      } catch (error) {
        toast.error('Please download the wallet on Chrome Extensions Store')
      }
    } else {
      toast("You've already connected to the wallet!", {
        icon: '✨',
      })
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
            {account === null ? 'Connect Wallet' : formatAddress(account?.getAddress())}
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
