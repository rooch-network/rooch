// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Progress } from '@/components/ui/progress'
import React, { useEffect, useState } from 'react'
import { SelfStakingCard } from './self-staking-card'
import { TokenInfo } from '@/pages/mint/util/get-token-info'
import Skeleton from 'react-loading-skeleton'

type MintDetailComponentsProps = {
  tokenInfo: TokenInfo | undefined,
  tokenAddress: string
}
export const MintDetailComponents: React.FC<MintDetailComponentsProps> = ({ tokenInfo, tokenAddress }) => {
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    if (!tokenInfo) {
      return
    }

    const timer = setTimeout(() => setProgress((Math.trunc((tokenInfo.endTime - tokenInfo.starTime) / (tokenInfo.endTime - tokenInfo.starTime) * 100)) || 0), 500)
    return () => clearTimeout(timer)
  }, [tokenInfo])

  return (
    <>
      {
        tokenInfo ?
          <div className="flex items-center justify-start w-full gap-2 text-muted-foreground dark:text-zinc-50 mt-2">
            <span className="text-sm">Process</span>
            <Progress value={progress} />
            <span className="text-sm flex items-center gap-1">
          <p className="font-semibold">{progress}</p>
          <p>%</p>
        </span>
          </div> : <Skeleton />
      }

      {
        <SelfStakingCard tokenAddress={tokenAddress} tokenInfo={tokenInfo} />
      }
    </>
  )
}
