import type {
  IndexerStateIDView,
  AnnotatedMoveStructView,
  PaginatedIndexerObjectStateViews,
} from '@roochnetwork/rooch-sdk';

import { useRef, useMemo, useState, useEffect } from 'react';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

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

function parseTokenName(type: string) {
  let tokenType = type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
  tokenType = tokenType.replace('>>', '');
  const tokenName = tokenType.split('::');
  return {
    type: tokenType,
    name: tokenName[tokenName.length - 1].replace('>>', ''),
  };
}

function parseTokenPairs(tokenPairs?: PaginatedIndexerObjectStateViews): AllLiquidityItemType[] {
  if (!tokenPairs) {
    return [];
  }

  const rowItem: AllLiquidityItemType[] = tokenPairs.data.map((item) => {
    const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
    const { type: xType, name: xName } = parseTokenName(xView.type);
    const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
    const { type: yType, name: yName } = parseTokenName(yView.type);
    const lpView = item.decoded_value!.value.coin_info as AnnotatedMoveStructView;
    return {
      id: item.id,
      creator: item.decoded_value!.value.creator as string,
      createAt: Number(item.created_at),
      lpTokenId: lpView.value.id as string,
      x: {
        id: xView.value.id as string,
        symbol: xName,
        type: xType,
      },
      y: {
        id: yView.value.id as string,
        symbol: yName,
        type: yType,
      },
    };
  });
  return rowItem;
}

export function useAllLiquidity(limit: number = 10): UseAllLiquidityReturn {
  const dex = useNetworkVariable('dex');

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

  const resolvedTokenPairs = useMemo(() => parseTokenPairs(tokenPairs), [tokenPairs]);

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
