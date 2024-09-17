import type { RoochClient, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

export type TokenInfo = {
  address: string;
  coin: {
    type: string;
    name: string;
    symbol: string;
    decimals: number;
  };
  assetTotalWeight: number;
  starTime: number;
  endTime: number;
  progress: number;
  releasePerSecond: number;
  finished: boolean;
};

function extractCoinInfoContent(input: string): string | null {
  const regex = /CoinInfo<([^>]+)>/;
  let match;

  if (input) {
    match = input.match(regex);
  }

  if (match && match[1]) {
    return match[1];
  }

  return null;
}

export async function getTokenInfo(
  client: RoochClient,
  address: string
): Promise<TokenInfo | undefined> {
  const data = await client.getStates({
    accessPath: `/resource/${address}/${address}::hold_farmer::FarmingAsset`,
    stateOption: {
      decode: true,
      showDisplay: true,
    },
  });
  const decode = (((data?.[0]?.decoded_value as any)?.value as any)?.value as any)?.value as any;
  const coinInfo = decode?.coin_info as AnnotatedMoveStructView;
  const coinId = coinInfo?.value?.id as string;

  const coinType = extractCoinInfoContent(coinInfo?.type)!;

  if (!coinId) {
    return undefined;
  }

  return client
    .getStates({
      accessPath: `/object/${coinId}`,
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })
    .then((sv) => {
      const coinView = (sv?.[0]?.decoded_value as any)?.value as any;
      const starTime = decode.start_time as number;
      const endTime = decode.end_time as number;
      const now = Date.now() / 1000;
      const progress =
        Math.trunc(
          ((now > endTime ? endTime - starTime : now - starTime) / (endTime - starTime)) * 100
        ) || 0;

      return {
        address,
        coin: {
          type: coinType,
          name: coinView.name,
          decimals: coinView.decimals,
          symbol: coinView.symbol,
        },
        starTime,
        endTime,
        progress,
        finished: endTime < now,
        assetTotalWeight: decode.asset_total_weight as number,
        releasePerSecond: decode.release_per_second as number,
      };
    });
}
