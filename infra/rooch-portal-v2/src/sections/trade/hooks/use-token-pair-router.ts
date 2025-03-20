import type { BalanceInfoView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import { useState, useEffect } from 'react';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { useNetworkVariable } from 'src/hooks/use-networks';
import { TokenGraph } from "../util/token-graph";

type TokenPairType = {
  x: BalanceInfoView;
  y: BalanceInfoView[];
};

export function useTokenPairRouter() {
  const client = useRoochClient();
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');
  const [tokenGraph, setTokenGraph] = useState<TokenGraph>(new TokenGraph());
  const [tokenPairsMap, setTokenPairsMap] = useState<Map<string, TokenPairType>>(new Map());
  const [tokenInfo, setTokenInfo] = useState<Map<string, TokenPairType>>(new Map());

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
      console.log('fetchInfo');
      const infos = tokenPairs.data?.map(async (item) => {
        const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
        const xType = parseType(xView);
        const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
        const yType = parseType(yView);

        const [xResult, yResult] = await Promise.all([
          client.getBalance({ owner: currentAddress!.toStr(), coinType: xType }),
          client.getBalance({ owner: currentAddress!.toStr(), coinType: yType }),
        ])

        return {
          x: xResult,
          y: yResult,
        }
      })

      await Promise.all(infos).then((result) => {
        const _pairMap = new Map<string, BalanceInfoView>();
        const _tokenGraph = new TokenGraph()
        const _tokenPairsMap = new Map<string, TokenPairType>();

        result.forEach((item) => {
          _tokenGraph.addPair([item.x.coin_type, item.y.coin_type]);
          _pairMap.set(item.x.coin_type, item.x);
          _pairMap.set(item.y.coin_type, item.y);
        });

        // skip path > 2
        const allPairs = _tokenGraph.findAllPairs().filter((item) => item.length <= 4)

        allPairs.forEach((item) => {
          // insert
          const x = _pairMap.get(item[0])!;
          const y = _pairMap.get(item[1])!;
          if (!_tokenPairsMap.has(x.coin_type)) {
            _tokenPairsMap.set(x.coin_type, {
              x,
              y:[y],
            });
          } else {
            _tokenPairsMap.get(x.coin_type)?.y.push(y);
          }

          // reverse
          const key1 = y.coin_type;
          if (!_tokenPairsMap.has(key1)) {
            _tokenPairsMap.set(key1, {
              x: y,
              y: [x],
            });
          } else {
            _tokenPairsMap.get(key1)?.y.push(x);
          }
        })

        setTokenInfo(_tokenPairsMap);
        setTokenGraph(_tokenGraph);
        setTokenPairsMap(_tokenPairsMap);
      });
    }

    fetchInfo();
  }, [tokenPairs, client, currentAddress, setTokenGraph, setTokenInfo, setTokenPairsMap]);

  return {
    tokenGraph,
    tokenPairsMap,
    tokenInfo,
    isPending: isPending && tokenInfo.size > 0,
  };
}
