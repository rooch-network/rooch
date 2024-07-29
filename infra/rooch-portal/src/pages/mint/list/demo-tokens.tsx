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

export const DemoTokens = () => {
  const navigate = useNavigate()
  const [tokens, setTokens] = useState<TokenInfo[]>([])

  const client = useRoochClient()
  const addresses = useNetworkVariable('mintAddress')

  useEffect(() => {
    let tokens: TokenInfo[] = []

    addresses.forEach((item) => {
      getTokenInfo(client, item).then((token) => {
        if (token) {
          tokens = tokens.concat(token)
          setTokens(tokens)
        }
      })
    })
  }, [client, addresses])

  return (
    <div className="rounded-lg border w-full">
      <SkeletonTheme baseColor="#27272A" highlightColor="#444">
        <Table>
          <TableHeader>
            <TableRow>
              {tokens.length > 0 ? (
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
            {tokens.length > 0 ? (
              tokens.map((token) => (
                <TableRow key={token.starTime}>
                  <TableCell className="font-medium">{token.coin.symbol}</TableCell>
                  <TableCell className="font-medium">{token.coin.name}</TableCell>
                  <TableCell className="font-medium">Self-Staking (without Lock)</TableCell>
                  <TableCell>
                    <div className="flex items-center justify-start gap-1">
                      <Progress value={token.progress} className="w-[60%]" />
                      <span>{token.progress}%</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-center">
                    <Button
                      variant="link"
                      size="sm"
                      onClick={() => {
                        navigate(`/mint/detail/${token.address}`)
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
