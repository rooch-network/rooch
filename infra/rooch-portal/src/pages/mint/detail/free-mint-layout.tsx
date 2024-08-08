// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react'
import { useParams } from 'react-router-dom'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

import { Transaction } from '@roochnetwork/rooch-sdk'
import {
  useCurrentWallet,
  useRoochClientQuery,
  UseSignAndExecuteTransaction,
} from '@roochnetwork/rooch-sdk-kit'

import { Badge } from '@/components/ui/badge'
//import { Progress } from '@/components/ui/progress.tsx'
import { DetailView } from '@/view/detail-view.tsx'
import { Button } from '@/components/ui/button'
import { formatTimestamp } from '@/utils/format.ts'
import { NFTSVG } from './free-mint-nft-svg.ts'
import { FMNFT } from '@/common/constant.ts'

export const FreeMintLayout = () => {
  const { address } = useParams()

  const { wallet } = useCurrentWallet()
  const [loading, setLoading] = useState<boolean>(false)
  const { mutateAsync: execute } = UseSignAndExecuteTransaction()
  const { data } = useRoochClientQuery('getStates', {
    accessPath: `/resource/${address}/${address}::og_nft::Config`,
  })
  const { data: nfts } = useRoochClientQuery('queryObjectStates', {
    filter: {
      // object_type: `${address}::og_nft::NFT`
      object_type_with_owner: {
        object_type: `${address}::og_nft::NFT`,
        filter_out: false,
        owner: wallet?.getRoochAddress().toHexAddress() || '',
      },
    },
    limit: '1',
    queryOption: {
      decode: false,
      showDisplay: false,
    },
  })
  console.log(wallet?.getBitcoinAddress().toStr())
  console.log(wallet?.getRoochAddress().toBech32Address())
  console.log(nfts)

  const handlerMint = () => {
    setLoading(true)
    const tx = new Transaction()
    tx.callFunction({
      target: `${address}::og_nft::mint`,
    })
    execute({
      transaction: tx,
    })
      .catch((why) => {
        console.log(why)
      })
      .then((r) => {
        console.log(r)
      })
      .finally(() => setLoading(false))
  }

  return (
    <SkeletonTheme baseColor="#27272A" highlightColor="#444">
      <DetailView title={'Back to Mint page'} back={'/mint'}>
        <div className="flex flex-col w-full items-start justify-start gap-3">
          <div className="flex flex-col w-full items-start justify-start gap-5 font-medium">
            <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36 flex flex-row items-center justify-start gap-3">
                <h1 className="text-3xl font-bold text-gray-800 dark:text-gray-50">
                  {FMNFT.symbol}
                </h1>
                <Badge
                  variant="outline"
                  className="rounded-full border-amber-500 text-amber-500 dark:border-amber-300 dark:text-amber-300 hover:bg-amber-500/10"
                >
                  In Progress
                </Badge>
              </div>
            </div>
            {data && data.length > 0 ? (
              <div className="flex items-center justify-start gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>Start Time :</span>
                </div>
                <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                  {formatTimestamp((data[0].created_at as any as number) / 1000)}
                </span>
              </div>
            ) : (
              <Skeleton width={150} />
            )}
            {data && data.length > 0 ? (
              <div className="flex items-center justify-start gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>End Time :</span>
                </div>
                <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                  ∞
                </span>
              </div>
            ) : (
              <Skeleton width={150} />
            )}
            <div dangerouslySetInnerHTML={{ __html: NFTSVG }} className="w-20"></div>
          </div>
        </div>
        <Button
          className="rounded-lg w-full mt-4 mb-2 md:mt-8"
          disabled={loading || nfts?.data != undefined}
          onClick={handlerMint}
        >
          {nfts?.data !== undefined ? 'Your Minted' : 'Mint'}
        </Button>
      </DetailView>
    </SkeletonTheme>
  )
}
