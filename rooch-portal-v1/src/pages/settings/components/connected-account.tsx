// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Copy, Check, CircleUser } from 'lucide-react'
import { formatAddress } from '@/utils/format.ts'
import { useCurrentAccount } from '@roochnetwork/rooch-sdk-kit'
import toast from 'react-hot-toast'

export const ConnectedAccount = () => {
  const account = useCurrentAccount()
  const [copied, setCopied] = useState(false)

  const handleClickCopy = () => {
    const textToCopy = account?.address || ''

    if (!account) {
      toast('Please connect your wallet', {
        icon: '✨',
      })
      return
    }

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
      <div className="relative p-40">
        <div className="absolute inset-0 bg-inherit bg-opacity-50 flex justify-center items-center">
          {account ? (
            <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
          ) : (
            <div className="flex flex-col items-center justify-center text-center">
              <CircleUser className="w-12 h-12 mb-4 text-gray-500" />
              <p className="text-xl text-zinc-500 font-semibold">Could not find your account</p>
              <p className="text-sm text-muted-foreground mt-2">Please connect to wallet.</p>
            </div>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Network Status</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">Networks</TableHead>
            <TableHead className="text-right">Address</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell className="font-medium">Bitcoin</TableCell>
            <TableCell className="text-right">
              <span className="flex items-center justify-end gap-0.5 text-muted-foreground">
                {account?.address ? (
                  <>
                    <p className="hidden md:block">{account.address}</p>
                    <p className="md:hidden">{formatAddress(account.address)}</p>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-6 h-6"
                      onClick={handleClickCopy}
                    >
                      {copied ? (
                        <Check className="w-3 h-3 text-green-500" />
                      ) : (
                        <Copy className="w-3 h-3" />
                      )}
                    </Button>
                  </>
                ) : (
                  <p>No account found</p>
                )}
              </span>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}
