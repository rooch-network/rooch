import toast from 'react-hot-toast'
import { Copy, RotateCcw } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'

import { formatAddress } from '@/utils/format'
import { useWalletStore } from '@roochnetwork/rooch-sdk-kit'
import { useNavigate } from 'react-router-dom'

export const ProfileCard = () => {
  const navigate = useNavigate()
  const account = useWalletStore((state) => state.currentAccount)

  const handleClickCopy = () => {
    if (!account) {
      toast('Please connect your wallet', {
        icon: '✨',
      })
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

  const handleRefreshPage = () => {
    navigate(0)
  }

  return (
    <Card className="relative overflow-hidden w-full border-none rounded-lg">
      <CardHeader className="absolute top-0 left-0 z-10 p-4 md:p-6 w-full">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-2xl md:text-3xl leading-tight text-white">
              Rooch Account #1
            </CardTitle>
            {/* <CardDescription className="text-wrap text-white/95 dark:text-white/70 text-xs md:text-sm">
              Manage Your Wallet Connections and Authorized Sessions.
            </CardDescription> */}
          </div>
          <div className="ml-4 flex flex-col items-end justify-start text-sm md:text-base">
            <span className="mt-1.5 text-white/95 dark:text-white/85 leading-3">Your balance</span>
            <span className="text-2xl md:text-3xl font-semibold text-white">$0.00</span>
            <Button
              variant="ghost"
              size="icon"
              // TODO: Improvement: only refresh the profile card
              onClick={handleRefreshPage}
              className="rounded-full h-8 w-8 hover:bg-transparent/15 transition-all"
            >
              <RotateCcw className="w-4 h-4 text-white" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div>
          <div className="bg-gradient-to-r bg-clip-padding font-black from-teal-500 via-purple-500 to-orange-500 text-white object-cover w-full h-28 md:h-32 opacity-80 select-none" />
        </div>
      </CardContent>
      <CardFooter className="flex justify-between relative pb-8 md:pb-12 px-4 md:px-6 dark:bg-primary-foreground h-full">
        <div className="absolute">
          <Avatar className="w-12 h-12 md:w-20 md:h-20">
            <AvatarImage src="https://github.com/shadcn.png" alt="Logic" />
            <AvatarFallback className="text-xl">LO</AvatarFallback>
          </Avatar>
        </div>
        <div className="absolute top-0 right-4 md:top-2 md:right-6">
          <div className="flex items-center justify-center gap-1 text-sm text-zinc-800/85 dark:text-white/85">
            <span>
              {account === null ? 'Wallet Address' : formatAddress(account?.getAddress())}
            </span>
            <Button
              variant="ghost"
              size="icon"
              onClick={handleClickCopy}
              className="rounded-full h-8 w-8 hover:bg-border transition-all"
            >
              <Copy className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </CardFooter>
    </Card>
  )
}
