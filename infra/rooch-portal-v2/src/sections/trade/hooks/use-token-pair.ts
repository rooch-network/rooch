import {
  useCurrentAddress,
  useRoochClient,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';
import { useEffect, useState } from 'react';
import type { AnnotatedMoveStructView, BalanceInfoView } from '@roochnetwork/rooch-sdk';
import { useNetworkVariable } from '../../../hooks/use-networks';

export type UseTokenPairReturn = {
  tokenPairs: Map<string, TokenPairType>;
  isPending: boolean;
};

type TokenPairType = {
  x: BalanceInfoView;
  y: BalanceInfoView[];
};

export function useTokenPair(): UseTokenPairReturn {
  const client = useRoochClient();
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');
  const [tokenPair, setTokenPair] = useState<Map<string, TokenPairType>>(new Map());

  const { data: tokenPairs, isPending } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type: `${dex.address}::swap::TokenPair`,
    },
    limit: '200',
  });

  useEffect(() => {
    if (!tokenPairs || !client || !currentAddress) {
      return;
    }

    const parseType = (coin: AnnotatedMoveStructView) => {
      const xType = coin.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
      return xType.replace('>>', '');
    };

    const fetchInfo = async () => {
      const infos = tokenPairs.data?.map(async (item) => {
        const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
        const xType = parseType(xView);
        const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
        const yType = parseType(yView);

        const [xResult, yResult] = await Promise.all([
          client.getBalance({ owner: currentAddress!.toStr(), coinType: xType }),
          client.getBalance({ owner: currentAddress!.toStr(), coinType: yType }),
        ]);

        return {
          x: xResult,
          y: yResult,
        };
      });

      await Promise.all(infos).then((result) => {
        const pairMap = new Map<string, TokenPairType>();

        result.forEach((item) => {
          // insert
          const key = item.x.symbol;
          if (!pairMap.has(key)) {
            pairMap.set(key, {
              x: item.x,
              y: [item.y],
            });
          } else {
            pairMap.get(key)?.y.push(item.y);
          }

          // reverse
          const key1 = item.y.symbol;
          if (!pairMap.has(key1)) {
            pairMap.set(key1, {
              x: item.y,
              y: [item.x],
            });
          } else {
            pairMap.get(key)?.y.push(item.x);
          }
        });

        setTokenPair(pairMap);
      });
    };

    fetchInfo();
  }, [client, currentAddress, tokenPairs]);

  return {
    tokenPairs: tokenPair,
    isPending: isPending && tokenPair.size > 0,
  };
}
