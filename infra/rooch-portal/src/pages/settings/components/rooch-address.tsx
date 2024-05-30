// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Copy, Check, CircleUser } from 'lucide-react'
import { formatAddress } from '@/utils/format.ts'
import { useCurrentAccount } from '@roochnetwork/rooch-sdk-kit'
import toast from 'react-hot-toast'

export const RoochAddress = () => {
  const account = useCurrentAccount()
  const [copied, setCopied] = useState(false)

  const handleClickCopy = () => {
    if (!account) {
      toast('Please connect your wallet', {
        icon: '✨',
      })
      return
    }

    const textToCopy = account.getRoochAddress()

    navigator.clipboard
      .writeText(textToCopy)
      .then(() => {
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      })
      .catch((err) => {
        console.error('Failed to copy:', err)
      })
  }

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center p-6 border rounded-lg bg-white dark:bg-zinc-900">
        <CircleUser className="w-12 h-12 mb-4 text-zinc-500" />
        <p className="text-xl text-zinc-500 font-semibold">Could not find your account</p>
        <p className="text-sm text-muted-foreground mt-2">Please connect to wallet.</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center justify-center p-6 border rounded-lg bg-white dark:bg-zinc-900">
      <div className="flex items-center justify-between w-full mb-4">
        <div className="flex flex-col">
          <h3 className="text-lg font-semibold text-zinc-900 dark:text-white">Rooch Network</h3>
          <p className="text-sm text-zinc-500 dark:text-zinc-400 text-wrap">
            This is your Rooch Address mapping from the wallet address
          </p>
        </div>
        <Button variant="ghost" size="icon" className="w-8 h-8" onClick={handleClickCopy}>
          {copied ? (
            <Check className="w-4 h-4 text-green-500" />
          ) : (
            <Copy className="w-4 h-4 text-zinc-500 dark:text-zinc-300" />
          )}
        </Button>
      </div>
      <div className="w-full bg-zinc-100 dark:bg-zinc-700 rounded-lg p-4 flex items-center justify-between">
        <span className="text-zinc-900 dark:text-white truncate md:hidden flex items-center justify-start">
          <img
            src="/rooch_white_logo.svg"
            alt="rooch logo"
            className="w-4 h-4 rounded-full bg-zinc-900 p-0.5 mr-2"
          />
          {formatAddress(account.getRoochAddress())}
        </span>
        <span className="text-zinc-900 dark:text-white truncate hidden md:flex items-center justify-start">
          <img
            src="/rooch_white_logo.svg"
            alt="rooch logo"
            className="w-4 h-4 rounded-full bg-zinc-900 p-0.5 mr-2"
          />
          {account.getRoochAddress()}
        </span>
      </div>
    </div>
  )
}
