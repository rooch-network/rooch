import type { BalanceInfoView } from '@roochnetwork/rooch-sdk';

import { useMemo } from 'react';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

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

function parseLPTokens(
  assetsList: BalanceInfoView[],
  dexAddress: string
): OwnerLiquidityItemType[] {
  if (!assetsList) {
    return [];
  }

  return assetsList
    .filter((item) => item.symbol.startsWith('RDexLP'))
    .map((item) => {
      const t = item.coin_type
        .replaceAll(' ', '')
        .replace(`${dexAddress}::swap::LPToken<`, '')
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
}

export function useOwnerLiquidity(): UseOwnerLiquidityReturn {
  const currentAddress = useCurrentAddress();
  const dex = useNetworkVariable('dex');

  const { data: assetsList, isPending } = useRoochClientQuery('getBalances', {
    owner: currentAddress?.toStr() || '',
  });

  const lpTokens = useMemo(
    () => parseLPTokens(assetsList?.data || [], dex.address),
    [assetsList, dex.address]
  );

  return {
    lpTokens,
    isPending,
  };
}
