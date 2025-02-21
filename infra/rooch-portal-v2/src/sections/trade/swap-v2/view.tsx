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
import { useTokenPair } from '../hooks/use-token-pair';

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

  const { tokenPairs } = useTokenPair();

  const availableFromCoins = useMemo(
    () =>
      Array.from(tokenPairs.values()).map((i) => ({
        ...i.x,
        icon_url: normalizeCoinIconUrl(i.x.icon_url),
        amount: '0',
      })),
    [tokenPairs]
  );

  const availableToCoins = useMemo(() => {
    if (!tokenPairs) {
      return [];
    }
    if (!fromCoinType) {
      return Array.from(tokenPairs.values()).map((i) => ({
        ...i.x,
        icon_url: normalizeCoinIconUrl(i.x.icon_url),
        amount: '0',
      }));
    }
    return (
      tokenPairs.get(fromCoinType as string)?.y.map((i) => ({
        ...i,
        icon_url: normalizeCoinIconUrl(i.icon_url),
        amount: '0',
      })) || []
    );
  }, [fromCoinType, tokenPairs]);

  const fromCoinInfo = useMemo(
    () => tokenPairs.get(fromCoinType as string)?.x,
    [tokenPairs, fromCoinType]
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
      const result = await client.executeViewFunction({
        target: `${dex.address}::router::get_amount_out`,
        args: [Args.u64(fixedFromCoinAmount)],
        typeArgs: [fromCoinInfo.coin_type, toCoinInfo.coin_type],
      });

      if (result.vm_status !== 'Executed') {
        toast.error('unknown error');
      }

      const toCoinAmount = result.return_values![0].decoded_value as string;
      const fixedToCoinAmount = fromDust(toCoinAmount, toCoinInfo?.decimals || 0);
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
  }, [fromCoinAmount, fromCoinInfo, toCoinInfo, client, dex.address]);

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
        />
      )}
    </DashboardContent>
  );
}
