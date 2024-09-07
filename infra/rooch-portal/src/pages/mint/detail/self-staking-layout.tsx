// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

import { BalanceInfoView } from '@roochnetwork/rooch-sdk'
import { useRoochClient, useCurrentWallet } from '@roochnetwork/rooch-sdk-kit'

import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress.tsx'
import { DetailView } from '@/view/detail-view.tsx'
import { TokenInfo, getTokenInfo } from '@/pages/mint/util/get-token-info'
import { ActionCard } from './components/stake-card'
import { formatTimestamp, formatCoin } from '@/utils/format.ts'

export const SelfStakingLayout = () => {
  const [tokenInfo, setTokenInfo] = useState<TokenInfo>()
  const [balance, setBalance] = useState<BalanceInfoView>()

  const { wallet } = useCurrentWallet()
  const { address } = useParams()
  const client = useRoochClient()

  useEffect(() => {
    getTokenInfo(client, address!).then((token) => {
      if (token) {
        setTokenInfo(token)
        client
          .getBalance({
            owner: wallet!.getRoochAddress().toHexAddress(),
            coinType: token.coin.type,
          })
          .then((result) => setBalance(result))
      }
    })
  }, [client, address, wallet])

  const [progress, setProgress] = useState(0)

  useEffect(() => {
    if (!tokenInfo) {
      return
    }
    const timer = setTimeout(() => setProgress(tokenInfo.progress), 500)
    return () => clearTimeout(timer)
  }, [tokenInfo])

  return (
    <SkeletonTheme baseColor="#27272A" highlightColor="#444">
      <DetailView title={'Back to Mint page'} back={'/mint'}>
        <div className="flex flex-col w-full items-start justify-start gap-3">
          <div className="flex flex-col w-full items-start justify-start gap-5 font-medium">
            <div className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              {tokenInfo ? (
                <div className="w-36 flex flex-row items-center justify-start gap-3">
                  <h1 className="text-3xl font-bold text-gray-800 dark:text-gray-50">
                    {tokenInfo.coin.symbol}
                  </h1>
                  <Badge
                    variant="outline"
                    className="rounded-full border-amber-500 text-amber-500 dark:border-amber-300 dark:text-amber-300 hover:bg-amber-500/10"
                  >
                    {tokenInfo.endTime < Math.floor(Date.now() / 1000) ? 'Finished' : 'In Progress'}
                  </Badge>
                </div>
              ) : (
                <Skeleton width={150} />
              )}
            </div>

            {tokenInfo ? (
              <div className="flex items-center justify-start gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>Start Time :</span>
                </div>
                <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                  {formatTimestamp(tokenInfo.starTime)}
                </span>
              </div>
            ) : (
              <Skeleton width={150} />
            )}
            {tokenInfo ? (
              <div className="flex items-center justify-start gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>End Time :</span>
                </div>
                <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                  {formatTimestamp(tokenInfo.endTime)}
                </span>
              </div>
            ) : (
              <Skeleton width={150} />
            )}
            {tokenInfo ? (
              <div className="flex items-center justify-start gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>Your Mint :</span>
                </div>
                <span className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                  {balance
                    ? formatCoin(Number(balance.balance), balance.decimals, balance.decimals)
                    : 0}
                </span>
              </div>
            ) : (
              <Skeleton width={150} />
            )}
            {tokenInfo ? (
              <div className="flex items-center justify-start w-full gap-2 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                <div className="w-24">
                  <span>Process :</span>
                </div>
                <div className="ml-4 w-full">
                  <Progress value={progress} />
                </div>
                <span className="text-sm flex items-center gap-1">
                  <p className="font-semibold">{progress}</p>
                  <p>%</p>
                </span>
              </div>
            ) : (
              <Skeleton />
            )}
          </div>
        </div>
        <ActionCard key="action-card" tokenAddress={address!} tokenInfo={tokenInfo} />
      </DetailView>
    </SkeletonTheme>
  )
}
