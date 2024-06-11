// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Button } from '@/components/ui/button'
import { Avatar } from '@/components/ui/avatar'

import Jazzicon, { jsNumberForAddress } from 'react-jazzicon'

import { formatAddress } from '@/utils/format'
import { useCurrentAccount } from '@roochnetwork/rooch-sdk-kit'

export const ProfileInfo = () => {
  const account = useCurrentAccount()

  return (
    <div className="h-12 w-full cursor-default flex items-center justify-start transition-all ease-in-out mt-2">
      <div className="flex items-center justify-start gap-x-3 w-full p-2 rounded-lg hover:bg-accent transition-all">
        <Avatar className="">
          {account ? (
            <Jazzicon diameter={55} seed={jsNumberForAddress(`0x${account.address}`)} />
          ) : (
            <Jazzicon diameter={55} seed={10000000} />
          )}
        </Avatar>
        <div className="h-full w-full flex flex-col items-start justify-center">
          <h3 className="text-base font-semibold text-zinc-500 dark:text-zinc-300">
            Rooch Account
          </h3>
          <div className="leading-none text-muted-foreground flex items-center justify-start font-normal text-xs sm:text-sm">
            {account ? <p>{formatAddress(account?.address)}</p> : <p>Connect to wallet</p>}
            <Button
              variant="ghost"
              size="icon"
              className="rounded-full h-4 w-4 transition-all hover:cursor-default ml-1"
            >
              <img
                src="/icon-btc.svg"
                alt="btc logo"
                className="w-4 h-4 rounded-full bg-gray-700 dark:bg-inherit"
              />
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
