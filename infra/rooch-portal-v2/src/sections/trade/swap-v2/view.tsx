'use client';

import type { CurveType, PoolVersion, InteractiveMode } from 'src/components/swap/types';

import BigNumber from 'bignumber.js';
import { useMemo, useState, useEffect, useCallback } from 'react';
import { Args, type BalanceInfoView, type AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { toDust, fromDust } from 'src/utils/number';

import { DashboardContent } from 'src/layouts/dashboard';

import Swap from 'src/components/swap/swap';
import { toast } from 'src/components/snackbar';

import SwapConfirmModal from './confirm-modal';

type TokenType = {
  id: string;
  type: string;
  name: string;
};

type TokenPairType = {
  x2y: boolean;
  x: TokenType;
  y: TokenType;
};

export default function SwapView() {
  const dex = useNetworkVariable('dex');
  const client = useRoochClient();
  const currentAddress = useCurrentAddress();

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

  const { data: userBalances } = useRoochClientQuery(
    'getBalances',
    {
      owner: currentAddress?.toStr() || '',
    },
    {
      refetchInterval: 5000,
    }
  );

  const assetsMap = useMemo(() => {
    const assetsMap = new Map<string, BalanceInfoView>();
    userBalances?.data.forEach((i) => {
      assetsMap.set(i.coin_type, {
        ...i,
      });
    });
    return assetsMap;
  }, [userBalances]);

  useEffect(() => {
    client
      .queryObjectStates({
        filter: {
          object_type: `${dex.address}::swap::TokenPair`,
        },
      })
      .then((result) => {
        const pair: TokenPairType[] = result.data.map((item) => {
          const xView = item.decoded_value!.value.balance_x as AnnotatedMoveStructView;
          let xType = xView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
          xType = xType.replace('>>', '');
          const xName = xType.split('::');
          const yView = item.decoded_value!.value.balance_y as AnnotatedMoveStructView;
          let yType = yView.type.replace('0x2::object::Object<0x3::coin_store::CoinStore<', '');
          yType = yType.replace('>>', '');
          const yName = yType.split('::');
          return {
            x2y: true,
            x: {
              id: xView.value.id as string,
              type: xType,
              name: xName[xName.length - 1].replace('>>', ''),
            },
            y: {
              id: yView.value.id as string,
              type: yType,
              name: yName[yName.length - 1].replace('>>', ''),
            },
          };
        });

        const pairMap = new Map<string, TokenPairType[]>();
        pair.forEach((p) => {
          const key = p.x.name;
          if (!pairMap.has(key)) {
            pairMap.set(key, []);
          }
          pairMap.get(key)!.push(p);

          const key1 = p.y.name;
          if (!pairMap.has(key1)) {
            pairMap.set(key1, []);
          }
          pairMap.get(key1)!.push({
            x2y: false,
            x: p.y,
            y: p.x,
          });
        });
      });
  }, [client, dex]);

  const fromCoinInfo = assetsMap.get(fromCoinType as string);
  const toCoinInfo = assetsMap.get(toCoinType as string);

  const fromCoin = useMemo(
    () => ({
      ...fromCoinInfo,
      balance: BigInt(fromCoinInfo?.balance || 0),
      amount: BigInt(toDust(fromCoinAmount.toString(), fromCoinInfo?.decimals || 0)),
      coinType: fromCoinInfo?.coin_type || '',
      decimals: fromCoinInfo?.decimals || 0,
      icon: `data:image/svg+xml;utf8,${encodeURIComponent(fromCoinInfo?.icon_url || '')}` || '',
      iconUrl: `data:image/svg+xml;utf8,${encodeURIComponent(fromCoinInfo?.icon_url || '')}` || '',
      name: fromCoinInfo?.name || '',
      symbol: fromCoinInfo?.symbol || '',
      price: 0,
    }),
    [fromCoinAmount, fromCoinInfo]
  );

  const toCoin = useMemo(
    () => ({
      ...toCoinInfo,
      balance: BigInt(toCoinInfo?.balance || 0),
      amount: BigInt(toDust(toCoinAmount.toString(), toCoinInfo?.decimals || 0)),
      coinType: toCoinInfo?.coin_type || '',
      decimals: toCoinInfo?.decimals || 0,
      icon: `data:image/svg+xml;utf8,${encodeURIComponent(toCoinInfo?.icon_url || '')}` || '',
      iconUrl: `data:image/svg+xml;utf8,${encodeURIComponent(toCoinInfo?.icon_url || '')}` || '',
      name: toCoinInfo?.name || '',
      symbol: toCoinInfo?.symbol || '',
      price: 0,
    }),
    [toCoinAmount, toCoinInfo]
  );

  const fetchToCoin = useCallback(async () => {
    if (
      fromCoinAmount === '0' ||
      fromCoinAmount === '' ||
      !fromCoin ||
      !toCoin ||
      !fromCoin.coinType ||
      !toCoin.coinType
    ) {
      return;
    }

    try {
      setLoading(true);
      const fixedFromCoinAmount = toDust(
        fromCoinAmount.toString().replaceAll(',', ''),
        assetsMap?.get(fromCoin.coinType)?.decimals || 0
      );
      const result = await client.executeViewFunction({
        target: `${dex.address}::router::get_amount_out`,
        args: [Args.u64(fixedFromCoinAmount)],
        typeArgs: [fromCoin.coinType, toCoin.coinType],
      });

      if (result.vm_status !== 'Executed') {
        toast.error('unknown error');
      }

      const toCoinAmount = result.return_values![0].decoded_value as string;
      const fixedToCoinAmount = fromDust(
        toCoinAmount,
        assetsMap?.get(toCoin.coinType)?.decimals || 0
      );
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
  }, [fromCoinAmount, fromCoin, toCoin, assetsMap, client, dex.address]);

  useEffect(() => {
    fetchToCoin();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fromCoinAmount, fromCoin, toCoin]);

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
        coins={Array.from(assetsMap.values()).map((i) => ({
          ...i,
          balance: BigInt(i.balance),
          amount: BigInt(0),
          coinType: i.coin_type,
          decimals: i.decimals,
          icon: `data:image/svg+xml;utf8,${encodeURIComponent(i.icon_url || '')}` || '',
          iconUrl: `data:image/svg+xml;utf8,${encodeURIComponent(i.icon_url || '')}` || '',
          price: 0,
        }))}
        fromCoin={fromCoin.coinType ? fromCoin : undefined}
        toCoin={toCoin.coinType ? toCoin : undefined}
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
          setFromCoinType(toCoin.coinType);
          setFromCoinAmount('0');
          setToCoinAmount('0');
          setToCoinType(fromCoin.coinType);
          fetchToCoin();
        }}
        onSwap={async (payload) => {
          const { fromCoin, toCoin, interactiveMode } = payload;
          setFromCoinType(fromCoin?.coinType);
          setFromCoinAmount(
            fromDust(fromCoin?.amount.toString() ?? '0', fromCoin?.decimals || 0).toString()
          );
          setToCoinType(toCoin?.coinType);
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
            type: fromCoin.coinType,
            decimal: fromCoin.decimals,
            balance: Number(fromCoin.balance),
            amount: fromCoinAmount,
          }}
          toCoin={{
            ...toCoin,
            type: toCoin.coinType,
            decimal: toCoin.decimals,
            balance: Number(toCoin.balance),
            amount: toCoinAmount,
          }}
        />
      )}
    </DashboardContent>
  );
}
