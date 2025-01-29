import { BalanceInfoView } from '@roochnetwork/rooch-sdk';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';
import { useMemo, useState, useCallback } from 'react';
import { useNetworkVariable } from 'src/hooks/use-networks';

export type OwnerLiquidityItemType = {
  x: {
    name: string;
    type: string;
  };
  y: {
    name: string;
    type: string;
  };
} & BalanceInfoView;

export type UseOwnerLiquidityReturn = {
  lpTokens: OwnerLiquidityItemType[];
  isPending: boolean;
};

export function useOwnerLiquidity(): UseOwnerLiquidityReturn {
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');

  const { data: assetsList, isPending } = useRoochClientQuery('getBalances', {
    owner: currentAddress?.toStr() || '',
  });

  const lpTokens = useMemo(() => {
    if (!assetsList) {
      return [];
    }
    const tokens: OwnerLiquidityItemType[] = assetsList!.data
      .filter((item) => item.symbol.startsWith('RDexLP'))
      .map((item) => {
        const t = item.coin_type
          .replaceAll(' ', '')
          .replace(`${dex.address}::swap::LPToken<`, '')
          .split(',');
        const x = t[0];
        const y = t[1].substring(0, t[1].length - 1);
        const xName = x.split('::');
        const yName = y.split('::');
        return {
          ...item,
          x: {
            type: x,
            name: xName[xName.length - 1],
          },
          y: {
            type: y,
            name: yName[yName.length - 1],
          },
        };
      })
      .sort((a, b) => b.fixedBalance - a.fixedBalance);
    return tokens;
  }, [assetsList, dex.address]);

  return {
    lpTokens,
    isPending,
  };
}
