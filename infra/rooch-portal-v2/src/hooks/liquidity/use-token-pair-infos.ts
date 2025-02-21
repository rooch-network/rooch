import type { AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';
import type { AllLiquidityItemType } from 'src/sections/trade/hooks/use-all-liquidity';

import { useQuery } from '@tanstack/react-query';
import { useRoochClient, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import { formatCoin } from 'src/utils/format-number';

export default function useTokenPairInfos(lpTokens: AllLiquidityItemType[]) {
  const client = useRoochClient();
  const address = useCurrentAddress();

  const {
    data: tokenPairInfos,
    isPending,
    isError,
    refetch,
  } = useQuery({
    queryKey: ['tokenPairInfos', address?.genRoochAddress().toStr()],
    queryFn: async () => {
      if (!address) {
        return new Map();
      }

      const tokenPairInfos = new Map();
      const promises = lpTokens.map(async (item) => {
        const [xResult, xCoinResult, yResult, yCoinResult] = await Promise.all([
          client.queryObjectStates({
            filter: {
              object_id: item.x.id,
            },
          }),
          client.getBalance({
            owner: address.toStr(),
            coinType: item.x.type,
          }),
          client.queryObjectStates({
            filter: {
              object_id: item.y.id,
            },
          }),
          client.getBalance({
            owner: address.toStr(),
            coinType: item.y.type,
          }),
        ]);

        const getBalance = (result: typeof xResult) =>
          (result.data[0].decoded_value?.value.balance as AnnotatedMoveStructView).value
            .value as string;

        const xBalance = getBalance(xResult);
        const yBalance = getBalance(yResult);

        return {
          id: item.id,
          pair: {
            x: formatCoin(Number(xBalance), xCoinResult.decimals, 2),
            y: formatCoin(Number(yBalance), yCoinResult.decimals, 2),
          },
        };
      });

      const results = await Promise.all(promises);
      results.forEach(({ id, pair }) => tokenPairInfos.set(id, pair));

      return tokenPairInfos;
    },
  });

  return { tokenPairInfos, isPending, isError, refetch };
}
