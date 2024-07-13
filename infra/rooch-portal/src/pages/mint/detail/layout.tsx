// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

import { useRoochClient } from '@roochnetwork/rooch-sdk-kit'

import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress.tsx'
import { DetailView } from '@/view/detail-view.tsx'
import { TokenInfo, getTokenInfo } from '@/pages/mint/util/get-token-info'
import { ActionCard } from './components/stake-card'

export const MintDetailLayout = () => {
  const [tokenInfo, setTokenInfo] = useState<TokenInfo>()

  const { address } = useParams()
  const client = useRoochClient()

  useEffect(() => {
    getTokenInfo(client, address!).then((token) => {
      if (token) {
        setTokenInfo(token)
      }
    })
  }, [client, address])

  const [progress, setProgress] = useState(0)

  useEffect(() => {
    if (!tokenInfo) {
      return
    }
    const now = Date.now() / 1000

    const timer = setTimeout(() => setProgress((Math.trunc((now > tokenInfo.endTime ? tokenInfo.endTime : now - tokenInfo.starTime) / (tokenInfo.endTime - tokenInfo.starTime) * 100)) || 0), 500)
    return () => clearTimeout(timer)
  }, [tokenInfo])

  return (
    <SkeletonTheme baseColor="#27272A" highlightColor="#444">
      <DetailView title={'Back to Mint page'} back={'/mint'}>
        <div>
          {tokenInfo ? <div className="flex items-center justify-start">
            <div className="flex flex-col items-start justify-start">
              <div className="flex flex-row items-center justify-start gap-3">
                <h1 className="text-3xl font-bold tracking-tight">{tokenInfo.coin.symbol}</h1>
                <Badge
                  variant="outline"
                  className="rounded-full border-amber-500 text-amber-500 dark:border-amber-300 dark:text-amber-300 hover:bg-amber-500/10"
                >
                  {tokenInfo.endTime < Math.floor((Date.now() / 1000)) ? 'Finished' : 'In Progress'}
                </Badge>
              </div>
              <p className="text-muted-foreground text-sm">Self-Staking</p>
            </div>
          </div> : <Skeleton width={150} />}
          {
            tokenInfo ?
              <div
                className="flex items-center justify-start w-full gap-2 text-muted-foreground dark:text-zinc-50 mt-2">
                <span className="text-sm">Process</span>
                <Progress value={progress} />
                <span className="text-sm flex items-center gap-1">
                  <p className="font-semibold">{progress}</p>
                  <p>%</p>
                </span>
              </div> : <Skeleton />
          }

          <ActionCard tokenAddress={address!} tokenInfo={tokenInfo} />

        </div>
      </DetailView>
    </SkeletonTheme>
  )
}
