// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
import { MousePointer2 } from 'lucide-react'
import { useNetworkVariable } from '@/networks'
import { useRoochClientQuery, useRoochClient } from '@roochnetwork/rooch-sdk-kit'
import { useEffect, useState } from 'react'
import { AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk'
import { formatTimestamp } from '@/utils/format'
import { useNavigate } from 'react-router-dom'

type TokenInfo =
  {
    coin: {
      name: string,
      symbol: string,
      decimals: number,
    }
    assetTotalWeight: number,
    starTime: number,
    endTime: number,
    releasePerSecond: number,
  }

export const Tokens = () => {
  const navigate = useNavigate()
  const [tokens, setTokens] = useState<TokenInfo[]>([])

  const client = useRoochClient()
  const s = useNetworkVariable('mintAddress')
  const { data } = useRoochClientQuery('getStates', {
    accessPath: `/resource/${s}/${s}::hold_farmer::FarmingAsset`,
    stateOption: {
      decode: true,
      showDisplay: true,
    },
  })

  useEffect(() => {

    if (data && data.length > 0) {
      console.log(data)
      const decode = (data[0].decoded_value as AnnotatedMoveStructView).value
      const coinId = (decode['coin_info'] as AnnotatedMoveStructView).value['id'] as string

      client.getStates({
        accessPath: `/object/${coinId}`,
        stateOption: {
          decode: true,
          showDisplay: true
        }
      }).then((sv) => {
        console.log(sv)
        const coinView = (((sv[0].decoded_value as any).value as any).value as any).value as any

        let token: TokenInfo = {
          coin: {
            name: coinView.name,
            decimals: coinView.decimals,
            symbol: coinView.symbol
          },
          starTime: decode['start_time'] as number,
          endTime: decode['end_time'] as number,
          assetTotalWeight: decode['asset_total_weight'] as number,
          releasePerSecond: decode['release_per_second'] as number,
        }

        setTokens([token])
      })
    }
  }, [client, data])

  return (
    <div className="rounded-lg border w-full">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[150px]">Name</TableHead>
            <TableHead>Start time</TableHead>
            <TableHead>End time</TableHead>
            <TableHead>Release per second</TableHead>
            <TableHead>Progress</TableHead>
            <TableHead className="text-center">Action</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {tokens.map((token) => (
            <TableRow key={token.starTime}>
              <TableCell className="font-medium">{token.coin.name}({token.coin.symbol})</TableCell>
              <TableCell className="font-medium">{formatTimestamp(token.starTime)}</TableCell>
              <TableCell className="font-medium">{formatTimestamp(token.endTime)}</TableCell>
              <TableCell className="font-medium">{token.releasePerSecond}</TableCell>
              <TableCell>
                <div className="flex items-center justify-start gap-1">
                  <Progress value={token.assetTotalWeight} className="w-[60%]" />
                  <span>{token.assetTotalWeight}%</span>
                </div>
              </TableCell>
              <TableCell className="text-center">
                <Button variant="link" size="sm" onClick={() => {
                  navigate('/mint/stake')
                }}>
                  <span className="flex items-center justify-center">
                    <MousePointer2 className="w-4 h-4 mr-1" />
                    Mint
                  </span>
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
