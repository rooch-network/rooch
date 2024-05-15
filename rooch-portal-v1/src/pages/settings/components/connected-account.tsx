// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
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
import { Copy, Unplug } from 'lucide-react'
import { formatAddress } from '@/utils/format.ts'
import { useCurrentAccount } from '@roochnetwork/rooch-sdk-kit'
import toast from 'react-hot-toast'

const networks = [
  {
    network: 'Bitcoin',
    address: 'bc1pr6mdxnc348lua02c32ad4uyyaw3kavjz4c8jzkh5ffvuq4ryvxhsf879j5',
    status: true,
  },
  // {
  //   network: 'Ethereum',
  //   address: '0xa4Baa73f17719173Ce5f31556349c5e1D5c8BB51',
  //   status: false,
  // },
]

export const ConnectedAccount = () => {
  const account = useCurrentAccount()

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
    <div className="rounded-lg border w-full">
      <Table>
        <TableCaption className="text-left pl-2 mb-2">Network Status</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">Networks</TableHead>
            <TableHead>Address</TableHead>
            <TableHead className="text-right">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {networks.map((network) => (
            <TableRow key={network.network}>
              <TableCell className="font-medium">{network.network}</TableCell>
              {network.network === 'Ethereum' ? (
                <>
                  {/* ETH Comming soon */}
                  <TableCell>
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      Coming soon ⌛️
                    </span>
                  </TableCell>
                  <TableCell></TableCell>
                </>
              ) : (
                <>
                  {/* BTC */}
                  <TableCell className="hidden md:table-cell">
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      {account?.address ? (
                        <>
                          <p>{formatAddress(account.address)}</p>
                          <Button
                            variant="ghost"
                            size="icon"
                            className=" w-6 h-6"
                            onClick={() => handleClickCopy('btc')}
                          >
                            <Copy className="w-3 h-3" />
                          </Button>
                        </>
                      ) : (
                        <p>No account found</p>
                      )}
                    </span>
                  </TableCell>
                  <TableCell className="md:hidden">
                    <span className="flex items-center justify-start gap-0.5 text-muted-foreground">
                      {account?.address ? (
                        <p>{formatAddress(account.address)}</p>
                      ) : (
                        <p>Connect your wallet</p>
                      )}
                      <Button variant="ghost" size="icon" className=" w-6 h-6">
                        <Copy className="w-3 h-3" />
                      </Button>
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    {account?.address ? (
                      <Button
                        variant="link"
                        size="sm"
                        className="text-red-500 dark:text-red-400 dark:hover:text-red-300 hover:text-red-600"
                      >
                        <Unplug className="h-4 w-4 mr-1" />
                        Disconnect
                      </Button>
                    ) : (
                      <Button
                        variant="link"
                        size="sm"
                        className="text-green-500 dark:text-green-400 dark:hover:text-green-300 hover:text-green-600"
                      >
                        <Unplug className="h-4 w-4 mr-1" />
                        Connect Wallet
                      </Button>
                    )}
                  </TableCell>
                </>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
