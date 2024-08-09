// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useEffect, useState } from 'react'
import { MousePointer2 } from 'lucide-react'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'
import { useNetworkVariable } from '@/networks'
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit'
import { useNavigate } from 'react-router-dom'
import { getTokenInfo, TokenInfo } from '@/pages/mint/util/get-token-info'
import { FMNFT } from '@/common/constant.ts'

type NFTInfo = {
  address: string
}

type MintType = {
  type: 'nft' | 'self_staking'
  name: string
  symbol: string
  distribution: string
  progress: number
  action: string
  data: NFTInfo | TokenInfo
}

export const DemoTokens = () => {
  const navigate = useNavigate()
  const [data, setData] = useState<MintType[]>([])

  const client = useRoochClient()
  const addresses = useNetworkVariable('mintAddress')

  useEffect(() => {
    let data: MintType[] = [
      {
        ...FMNFT,
      } as MintType,
    ]

    addresses.forEach((item) => {
      getTokenInfo(client, item)
        .then((token) => {
          if (token) {
            data = data.concat({
              type: 'self_staking',
              action: `/mint/self/staking/${token.address}`,
              name: token.coin.name,
              symbol: token.coin.symbol,
              distribution: 'Self-Staking (without Lock)',
              progress: token.progress,
              data: token,
            })
          }
        })
        .finally(() => setData(data))
    })
  }, [client, addresses])

  return (
    <div className="rounded-lg border w-full">
      <SkeletonTheme baseColor="#27272A" highlightColor="#444">
        <Table>
          <TableHeader>
            <TableRow>
              {data.length > 0 ? (
                <>
                  <TableHead className="w-[150px]">Symbol</TableHead>
                  <TableHead className="w-[150px]">Name</TableHead>
                  <TableHead className="w-[200px]">Distribution Mechanism</TableHead>
                  <TableHead>Progress</TableHead>
                  <TableHead className="text-center w-[250px]">Action</TableHead>
                </>
              ) : (
                <TableHead className="w-full">
                  <Skeleton />
                </TableHead>
              )}
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.length > 0 ? (
              data.map((item) => (
                <TableRow key={item.name}>
                  <TableCell className="font-medium">{item.symbol}</TableCell>
                  <TableCell className="font-medium">{item.name}</TableCell>
                  <TableCell className="font-medium">{item.distribution}</TableCell>
                  <TableCell>
                    <div className="flex items-center justify-start gap-1">
                      <Progress
                        value={item.progress === -1 ? 100 : item.progress}
                        className="w-[60%]"
                      />
                      <span>{item.progress === -1 ? '∞' : `${item.progress}%`}</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-center">
                    <Button
                      variant="link"
                      size="sm"
                      onClick={() => {
                        navigate(item.action)
                      }}
                    >
                      <span className="flex items-center justify-center">
                        <MousePointer2 className="w-4 h-4 mr-1" />
                        Mint
                      </span>
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow key="loading">
                <TableCell className="font-medium w-full">
                  <Skeleton />
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </SkeletonTheme>
    </div>
  )
}
