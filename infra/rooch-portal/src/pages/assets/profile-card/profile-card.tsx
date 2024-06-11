// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import toast from 'react-hot-toast'
// import { RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Avatar } from '@/components/ui/avatar'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'

import { formatAddress } from '@/utils/format'
import { useCurrentAccount } from '@roochnetwork/rooch-sdk-kit'
// import { useNavigate } from 'react-router-dom'
import Jazzicon, { jsNumberForAddress } from 'react-jazzicon'

export const ProfileCard = () => {
  // const navigate = useNavigate()
  const account = useCurrentAccount()

  // TODO: handleClickCopy
  const handleClickCopy = (accountType: string) => {
    let textToCopy: string | null = ''

    if (!account) {
      toast('Please connect your wallet', {
        icon: '✨',
      })
      return
    }

    if (accountType === 'btc') {
      textToCopy = account.address
    } else if (accountType === 'rooch') {
      textToCopy = account.getRoochAddress()
    }

    if (textToCopy) {
      navigator.clipboard
        .writeText(textToCopy)
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

  return (
    <Card className="relative overflow-hidden w-full border-none rounded-lg">
      <CardHeader className="absolute top-0 left-0 z-10 p-4 md:p-6 w-full">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle>
              <div className="flex flex-col items-start justify-start">
                <h3 className="text-2xl md:text-3xl leading-tight text-white">Rooch Account</h3>
              </div>
            </CardTitle>
          </div>
          {/*<div className="ml-4 flex flex-col items-end justify-start text-sm md:text-base">*/}
          {/*  <span className="mt-1.5 text-white/95 dark:text-white/85 leading-3">Your balance</span>*/}
          {/*  <span className="text-2xl md:text-3xl font-semibold text-white">$0.00</span>*/}
          {/*  <Button*/}
          {/*    variant="ghost"*/}
          {/*    size="icon"*/}
          {/*    onClick={handleRefreshPage}*/}
          {/*    className="rounded-full h-8 w-8 hover:bg-transparent/15 transition-all"*/}
          {/*  >*/}
          {/*    <RotateCcw className="w-4 h-4 text-white" />*/}
          {/*  </Button>*/}
          {/*</div>*/}
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div>
          <div className="bg-gradient-to-r bg-clip-padding font-black from-teal-500 via-purple-500 to-orange-500 text-white object-cover w-full h-28 md:h-32 opacity-80 select-none" />
        </div>
      </CardContent>
      <CardFooter className="flex justify-between relative pb-8 md:pb-12 px-4 md:px-6 dark:bg-primary-foreground h-full">
        <div className="absolute">
          <Avatar className="hidden md:inline">
            {account ? (
              <Jazzicon diameter={80} seed={jsNumberForAddress(`0x${account.address}`)} />
            ) : (
              <Jazzicon diameter={80} seed={10000000} />
            )}
          </Avatar>
          <Avatar className="inline md:hidden">
            {account ? (
              <Jazzicon diameter={55} seed={jsNumberForAddress(account.address)} />
            ) : (
              <Jazzicon diameter={55} seed={10000000} />
            )}
          </Avatar>
        </div>
        <div className="absolute top-1 right-4 md:top-3 md:right-6">
          <div className="flex items-center justify-center gap-1 text-zinc-800/85 dark:text-white/85">
            {/* Rooch Address */}
            {/*<div*/}
            {/*  className="leading-none text-muted-foreground dark:text-white/85 flex items-center justify-start font-normal text-xs sm:text-sm hover:cursor-pointer"*/}
            {/*  onClick={() => handleClickCopy('rooch')}*/}
            {/*>*/}
            {/*  <Button*/}
            {/*    variant="ghost"*/}
            {/*    size="icon"*/}
            {/*    className="h-6 w-6 bg-inherit hover:bg-inherit transition-all"*/}
            {/*  >*/}
            {/*    <img*/}
            {/*      src="/rooch_white_logo.svg"*/}
            {/*      alt="rooch logo"*/}
            {/*      className="w-4 h-4 rounded-full bg-gray-700 p-0.5 dark:bg-gray-700"*/}
            {/*    />*/}
            {/*  </Button>*/}
            {/*  <span className="text-muted-foreground">*/}
            {/*    {account?.getRoochAddress() ? (*/}
            {/*      <p>{formatAddress(account?.getRoochAddress() as string)}</p>*/}
            {/*    ) : (*/}
            {/*      <p>Rooch Address</p>*/}
            {/*    )}*/}
            {/*  </span>*/}
            {/*</div>*/}

            {/* Wallet Address */}
            <div
              className="leading-none text-white/85 flex items-center justify-start font-normal text-xs sm:text-sm ml-3 hover:cursor-pointer"
              onClick={() => handleClickCopy('btc')}
            >
              <Button variant="ghost" size="icon" className="h-6 w-6 bg-inherit hover:bg-inherit">
                <img src="/icon-btc.svg" alt="btc logo" className="w-4 h-4" />
              </Button>
              <span className="text-muted-foreground">
                {account === null ? 'Wallet Address' : formatAddress(account?.address)}
              </span>
            </div>
          </div>
        </div>
      </CardFooter>
    </Card>
  )
}
