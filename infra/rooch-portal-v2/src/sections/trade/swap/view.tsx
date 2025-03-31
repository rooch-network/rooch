'use client';

import type { UserCoin, CurveType, PoolVersion, InteractiveMode } from 'src/components/swap/types';

import BigNumber from 'bignumber.js';
import { Args } from '@roochnetwork/rooch-sdk';
import { useMemo, useState, useCallback } from 'react';
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { toDust, fromDust } from 'src/utils/number';

import { DashboardContent } from 'src/layouts/dashboard';

import Swap from 'src/components/swap/swap';
import { toast } from 'src/components/snackbar';

import SwapConfirmModal from './confirm-modal';
import { useTokenPairRouter } from '../hooks/use-token-pair-router';

const normalizeCoinIconUrl = (onChainCoinIconUrl?: string | null) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(onChainCoinIconUrl || '')}`;

export default function SwapView() {
  const dex = useNetworkVariable('dex');
  const client = useRoochClient();

  const [loading, setLoading] = useState(false);
  const [curve, setCurve] = useState<CurveType>('uncorrelated');
  const [version, setVersion] = useState<PoolVersion>(0);
  const [slippage, setSlippage] = useState(0.005);
  const [openSwapModal, setOpenSwapModal] = useState(false);
  const [price, setPrice] = useState('');

  const [interactiveMode, setInteractiveMode] = useState<InteractiveMode>('from');

  const [fromCoinType, setFromCoinType] = useState<string | undefined>();
  const [toCoinType, setToCoinType] = useState<string>();
  const [fromCoinAmount, setFromCoinAmount] = useState<string>('0');
  const [toCoinAmount, setToCoinAmount] = useState<string>('');

  const [currentPath, setCurrentPath] = useState<string[] | null>(null);

  const { tokenPairsMap, tokenGraph, refetchTokenPairs } = useTokenPairRouter();
  console.log('🚀 ~ view.tsx:47 ~ SwapView ~ tokenPairsMap:', tokenPairsMap);

  const availableFromCoins = useMemo(
    () =>
      Array.from(tokenPairsMap.values()).map((i) => ({
        ...i.x,
        icon_url: normalizeCoinIconUrl(i.x.icon_url),
        amount: '0',
      })),
    [tokenPairsMap]
  );

  const availableToCoins = useMemo(() => {
    if (!tokenPairsMap) {
      return [];
    }
    if (!fromCoinType) {
      return Array.from(tokenPairsMap.values()).map((i) => ({
        ...i.x,
        icon_url: normalizeCoinIconUrl(i.x.icon_url),
        amount: '0',
      }));
    }
    return (
      tokenPairsMap.get(fromCoinType as string)?.y.map((i) => ({
        ...i,
        icon_url: normalizeCoinIconUrl(i.icon_url),
        amount: '0',
      })) || []
    );
  }, [fromCoinType, tokenPairsMap]);

  const fromCoinInfo = useMemo(
    () => tokenPairsMap.get(fromCoinType as string)?.x,
    [tokenPairsMap, fromCoinType]
  );

  const toCoinInfo = useMemo(
    () => availableToCoins.find((i) => i.coin_type === toCoinType),
    [availableToCoins, toCoinType]
  );

  const fromCoin = useMemo<UserCoin | undefined>(
    () =>
      fromCoinInfo
        ? {
            ...fromCoinInfo,
            icon_url: normalizeCoinIconUrl(fromCoinInfo.icon_url),
            amount: fromCoinAmount,
          }
        : undefined,
    [fromCoinAmount, fromCoinInfo]
  );

  const toCoin = useMemo<UserCoin | undefined>(
    () =>
      toCoinInfo
        ? {
            ...toCoinInfo,
            amount: toCoinAmount,
          }
        : undefined,
    [toCoinAmount, toCoinInfo]
  );

  const fetchToCoin = useCallback(async () => {
    if (fromCoinAmount === '0' || fromCoinAmount === '' || !fromCoinInfo || !toCoinInfo) {
      setToCoinAmount('');
      return;
    }
    try {
      setLoading(true);
      const fixedFromCoinAmount = toDust(
        fromCoinAmount.replaceAll(',', ''),
        fromCoinInfo.decimals || 0
      );
      const path = tokenGraph.findPath(fromCoinInfo.coin_type, toCoinInfo.coin_type);

      if (!path || path.length < 2) {
        toast.error('No valid swap path found');
        return;
      }

      // Start with the initial amount
      let currentAmount = fixedFromCoinAmount;

      // For each step in the path, call get_amount_out
      // eslint-disable-next-line no-plusplus
      for (let i = 0; i < path.length - 1; i++) {
        const fromToken = path[i];
        const toToken = path[i + 1];

        // eslint-disable-next-line no-await-in-loop
        const result = await client.executeViewFunction({
          target: `${dex.address}::router::get_amount_out`,
          args: [Args.u64(currentAmount)],
          typeArgs: [fromToken, toToken],
        });

        console.log(`Step ${i + 1}: ${fromToken} -> ${toToken}, result:`, result);

        if (result.vm_status !== 'Executed') {
          toast.error(`Error in swap step ${i + 1}`);
          return;
        }

        // Update the amount for the next step
        currentAmount = BigInt(result.return_values![0].decoded_value as string);
      }

      setCurrentPath(path);

      const fixedToCoinAmount = fromDust(currentAmount, toCoinInfo?.decimals || 0);
      const ratio = BigNumber(fixedToCoinAmount).div(fromCoinAmount);
      const fixedRatio = ratio.toFixed(8, 1);
      const finalRatio = ratio.isInteger() ? ratio.toFixed(0) : fixedRatio;
      setPrice(finalRatio);
      setToCoinAmount(fixedToCoinAmount.toString());
    } catch (e) {
      console.log(e);
    } finally {
      setLoading(false);
    }
  }, [fromCoinAmount, fromCoinInfo, toCoinInfo, client, dex.address, tokenGraph]);

  return (
    <DashboardContent maxWidth="xl" className="items-center">
      <Swap
        isValid
        hiddenValue
        easyMode
        swapHeaderTitle="Swap"
        containerProps={{
          sx: {
            maxWidth: '620px',
          },
        }}
        loading={loading}
        availableFromCoins={availableFromCoins}
        availableToCoins={availableToCoins}
        fromCoin={fromCoinType ? fromCoin : undefined}
        toCoin={toCoinType ? toCoin : undefined}
        interactiveMode={interactiveMode}
        canSelectCurve={false}
        curve={curve}
        convertRate={Number(price)}
        platformFeePercent={0}
        priceImpact={0}
        priceImpactSeverity="normal"
        version={version}
        slippagePercent={slippage}
        onSlippageChange={(slippage: number) => {
          setSlippage(slippage);
        }}
        onCurveTypeChange={(curveType: CurveType) => setCurve(curveType)}
        onVersionChange={(version: PoolVersion) => setVersion(version)}
        onSwitch={() => {
          setFromCoinType(toCoinInfo?.coin_type);
          setFromCoinAmount('0');
          setToCoinAmount('0');
          setToCoinType(fromCoinInfo?.coin_type);
          fetchToCoin();
        }}
        onSwap={async (payload) => {
          const { fromCoin, toCoin, interactiveMode } = payload;
          setFromCoinType(fromCoin?.coin_type);
          setFromCoinAmount(
            fromDust(fromCoin?.amount.toString() ?? '0', fromCoinInfo?.decimals || 0).toString() ||
              '0'
          );
          setToCoinType(toCoin?.coin_type);
          setInteractiveMode(interactiveMode);
          fetchToCoin();
        }}
        onPreview={async () => {
          setOpenSwapModal(true);
        }}
        onPropose={async () => {}}
      />
      {fromCoin && toCoin && (
        <SwapConfirmModal
          slippage={slippage}
          open={openSwapModal}
          path={currentPath || []}
          onClose={() => setOpenSwapModal(false)}
          fromCoin={{
            ...fromCoin,
            coin_type: fromCoin.coin_type,
            decimals: fromCoin.decimals,
            balance: fromCoin.balance.toString(),
            amount: fromCoinAmount,
          }}
          toCoin={{
            ...toCoin,
            coin_type: toCoin.coin_type,
            decimals: toCoin.decimals,
            balance: toCoin.balance.toString(),
            amount: toCoinAmount,
          }}
          onSuccess={() => {
            refetchTokenPairs();
          }}
        />
      )}
    </DashboardContent>
  );
}
