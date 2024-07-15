// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { RoochClient, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk'

export type TokenInfo =
  {
    address: string,
    coin: {
      name: string,
      symbol: string,
      decimals: number,
    }
    assetTotalWeight: number,
    starTime: number,
    endTime: number,
    progress: number,
    releasePerSecond: number,
  }

export async function getTokenInfo(client: RoochClient, address: string): Promise<TokenInfo | undefined> {
  const data = await client.getStates({
    accessPath: `/resource/${address}/${address}::hold_farmer::FarmingAsset`,
    stateOption: {
      decode: true,
      showDisplay: true,
    },
  })
  const decode = (((data[0].decoded_value as any).value as any).value as any).value as any
  const coinId = (decode['coin_info'] as AnnotatedMoveStructView).value['id'] as string

  return client.getStates({
    accessPath: `/object/${coinId}`,
    stateOption: {
      decode: true,
      showDisplay: true
    }
  }).then((sv) => {
    const coinView = (sv[0].decoded_value as any).value as any
    const starTime= decode['start_time'] as number
    const endTime = decode['end_time'] as number
    const now = Date.now() / 1000
    const progress = Math.trunc((now > endTime ? endTime - starTime : now - starTime) / (endTime - starTime) * 100) || 0

    return {
      address: address,
      coin: {
        name: coinView.name,
        decimals: coinView.decimals,
        symbol: coinView.symbol
      },
      starTime,
      endTime,
      progress,
      assetTotalWeight: decode['asset_total_weight'] as number,
      releasePerSecond: decode['release_per_second'] as number,
    }
  })

  return undefined
}