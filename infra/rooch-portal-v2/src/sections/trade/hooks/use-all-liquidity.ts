import type { IndexerStateIDView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState, useEffect } from 'react';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { useNetworkVariable } from 'src/hooks/use-networks';

export type AllLiquidityItemType = {
  id: string;
  createAt: number;
  x: {
    id: string;
    symbol: string;
    type: string;
  };
  y: {
    id: string;
    symbol: string;
    type: string;
  };
  lpTokenId: string;
  creator: string;
};

export type UseAllLiquidityReturn = {
  hasNext: boolean;
  index: number;
  paginate: (index: number) => void;
  lpTokens: AllLiquidityItemType[];
  isPending: boolean;
};

export function useAllLiquidity(limit: number = 10): UseAllLiquidityReturn {
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');
  const client = useRoochClient();

  const [paginationModel, setPaginationModel] = useState({ index: 1, limit });
  const mapPageToNextCursor = useRef<{ [page: number]: IndexerStateIDView | null }>({});

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.index - 1],
      limit: paginationModel.limit.toString(),
    }),
    [paginationModel]
  );

  const paginate = (index: number): void => {
    if (index < 0) {
      return;
    }
    setPaginationModel({
      ...paginationModel,
      index,
    });
  };

  const { data: tokenPairs, isPending } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type: `${dex.address}::swap::TokenPair`,
    },
    cursor: queryOptions.cursor,
    limit: queryOptions.limit,
    queryOption: {
      showDisplay: true,
    },
  });

  const resolvedTokenPairs = useMemo(() => {
    if (!tokenPairs) {
      return [];
    }

    const rowItme: AllLiquidityItemType[] = tokenPairs!.data.map((item) => {
      const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
      let xType = xView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
      xType = xType.replace('>>', '');
      const xName = xType.split('::');
      const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
      let yType = yView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
      yType = yType.replace('>>', '');
      const yName = yType.split('::');
      const lpView = item.decoded_value!.value.coin_info as AnnotatedMoveStructView;
      return {
        id: item.id,
        creator: item.decoded_value!.value.creator as string,
        createAt: Number(item.created_at),
        lpTokenId: lpView.value.id as string,
        x: {
          id: xView.value.id as string,
          symbol: xName[xName.length - 1].replace('>>', ''),
          type: xType,
        },
        y: {
          id: yView.value.id as string,
          symbol: yName[xName.length - 1].replace('>>', ''),
          type: yType,
        },
      };
    });

    return rowItme;
  }, [tokenPairs]);

  useEffect(() => {
    if (!tokenPairs) {
      return;
    }
    if (tokenPairs.has_next_page) {
      mapPageToNextCursor.current[paginationModel.index] = tokenPairs.next_cursor ?? null;
    }
  }, [paginationModel, tokenPairs]);

  return {
    hasNext: tokenPairs?.has_next_page || false,
    index: paginationModel.index,
    paginate,
    lpTokens: resolvedTokenPairs,
    isPending,
  };
}
