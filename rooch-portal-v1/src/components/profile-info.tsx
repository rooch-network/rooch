import { Button } from '@/components/ui/button'
import { Avatar } from '@/components/ui/avatar'

import Jazzicon, { jsNumberForAddress } from 'react-jazzicon'

import { formatAddress } from '@/utils/format'
import { useWalletStore } from '@roochnetwork/rooch-sdk-kit'
import { useWalletAccountStore } from '@/store/useWalletAccountStore'

export const ProfileInfo = () => {
  const account = useWalletStore((state) => state.currentAccount)
  const { roochAddress } = useWalletAccountStore()

  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-12 w-full cursor-pointer flex items-center justify-start transition-all ease-in-out"
    >
      <div className="flex items-center justify-start gap-x-3">
        <Avatar className="">
          {account ? (
            <Jazzicon diameter={55} seed={jsNumberForAddress(account.address)} />
          ) : (
            <Jazzicon diameter={55} seed={10000000} />
          )}
        </Avatar>
        <div className="h-full w-full flex flex-col items-start justify-center">
          <h3 className="text-base font-semibold text-zinc-500 dark:text-zinc-300">
            Rooch Network
          </h3>
          <div className="leading-none text-white/85 flex items-center justify-start font-normal text-xs sm:text-sm hover:cursor-pointer">
            {roochAddress ? <p>{formatAddress(roochAddress as string)}</p> : <p>Rooch Address</p>}
            <Button variant="ghost" size="icon" className="rounded-full h-4 w-4 transition-all">
              <img src="/rooch_white_logo.svg" alt="rooch logo" className="w-3 h-3" />
            </Button>
          </div>
        </div>
      </div>
    </Button>
  )
}
