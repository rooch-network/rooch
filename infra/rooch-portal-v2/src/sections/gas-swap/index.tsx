'use client';

import { Stack } from '@mui/material';
import { useState, ReactNode, useEffect } from 'react';
import Swap from 'src/components/swap/swap';
import {
  UserCoin,
  InteractiveMode,
  DEFAULT_SLIPPAGE,
  CurveType,
  PoolVersion,
} from 'src/components/swap/types';
import { sleep } from 'src/utils/common';
import { toBigNumber, toDust, fromDust } from 'src/utils/number';

const demoCoins = [
  {
    coinType: 'usdt',
    decimals: 6,
    symbol: 'USDT',
    name: 'Tether USD',
    icon: 'https://liquidswap.com/assets/usdt-cddba428.svg',
    balance: 0n,
    amount: 0n,
    price: 1,
  },
  {
    coinType: 'usdc',
    decimals: 6,
    symbol: 'USDC',
    name: 'USD Coin',
    icon: 'https://liquidswap.com/assets/usdc-913adf09.svg',
    balance: 11000000n,
    amount: 0n,
    price: 1,
  },
];

export default function GasSwapOverview() {
  const [loading, setLoading] = useState<boolean>(false);
  const [coins, setCoins] = useState<UserCoin[]>([]);
  const [fromCoin, setFromCoin] = useState<UserCoin>();
  const [toCoin, setToCoin] = useState<UserCoin>();
  const [interactiveMode, setInteractiveMode] = useState<InteractiveMode>('from');
  const [slippage, setSlippage] = useState<number>(DEFAULT_SLIPPAGE);
  const [canSelectCurve, setCanSelectCurve] = useState<boolean>(false);
  const [curve, setCurve] = useState<CurveType>('uncorrelated');
  const [warning, setWarning] = useState<ReactNode>();
  const [convertRate, setConvertRate] = useState<number>();
  const [platformFeePercent] = useState<number>(0.003);
  const [platformFeeAmount, setPlatformFeeAmount] = useState<number>();
  const [msafeFeePercent] = useState<number>(0.002);
  const [msafeFeeAmount, setMsafeFeeAmount] = useState<number>();
  const [swapAmount, setSwapAmount] = useState(0n);
  const [slippageAmount, setSlippageAmount] = useState<number>();
  const [canSelectVersion, setCanSelectVersion] = useState<boolean>(false);
  const [version, setVersion] = useState<PoolVersion>(0);

  useEffect(() => {
    const timer = setTimeout(() => {
      setCoins(demoCoins);
      setFromCoin(demoCoins[0]);
    }, 500);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    const execute = async () => {
      if (fromCoin && toCoin) {
        setLoading(true);
        await fetchFee();
        await fetchRate();

        setCanSelectVersion(true);
        setCurve('uncorrelated');
        setCanSelectCurve(true);
        setWarning(
          'Caution: make sure the pair you are trading should be stable or uncorrelated. i.e USDC/USDT is stable and USDC/BTC is uncorrelated'
        );
        setLoading(false);
      }
    };

    execute();
  }, [fromCoin?.coinType, fromCoin?.amount, toCoin?.coinType, toCoin?.amount, interactiveMode]);

  const fetchRate = async (): Promise<void> => {
    if (fromCoin && toCoin) {
      await sleep(500);
      if (interactiveMode === 'from' && toBigNumber(fromCoin.amount).gt(0)) {
        setToCoin({
          ...toCoin,
          amount:
            fromCoin.coinType === 'apt'
              ? toDust(
                  fromDust(fromCoin.amount, fromCoin.decimals)
                    .times(7.12)
                    .decimalPlaces(toCoin.decimals),
                  toCoin.decimals
                )
              : toDust(
                  fromDust(fromCoin.amount, fromCoin.decimals)
                    .div(7.11)
                    .decimalPlaces(toCoin.decimals),
                  toCoin.decimals
                ),
        });
      } else if (interactiveMode === 'to' && toBigNumber(toCoin.amount).gt(0)) {
        const coin = {
          ...fromCoin,
          amount:
            toCoin.coinType === 'apt'
              ? toDust(
                  fromDust(toCoin.amount, toCoin.decimals)
                    .times(7.12)
                    .decimalPlaces(fromCoin.decimals),
                  fromCoin.decimals
                )
              : toDust(
                  fromDust(toCoin.amount, toCoin.decimals)
                    .div(7.11)
                    .decimalPlaces(fromCoin.decimals),
                  fromCoin.decimals
                ),
        };
        setFromCoin(coin);
      }
    }
  };

  const fetchFee = async (): Promise<void> => {
    await sleep(500);
    setConvertRate(1);
    setPlatformFeeAmount(1);
    setMsafeFeeAmount(1);
    setSlippageAmount(1);
    setSwapAmount(1n);
  };

  return (
    <Stack className="w-full justify-center items-center">
      <Stack className="w-3/4 max-w-[600px]">
        <Swap
          loading={loading}
          coins={coins}
          fromCoin={fromCoin}
          toCoin={toCoin}
          interactiveMode={interactiveMode}
          slippagePercent={slippage}
          canSelectCurve={canSelectCurve}
          curve={curve}
          warning={warning}
          convertRate={convertRate}
          platformFeePercent={platformFeePercent}
          platformFeeAmount={platformFeeAmount}
          msafeFeePercent={msafeFeePercent}
          msafeFeeAmount={msafeFeeAmount}
          slippageAmount={slippageAmount}
          swapAmount={swapAmount}
          priceImpact={0.1}
          priceImpactSeverity="normal"
          canSelectVersion={canSelectVersion}
          version={version}
          onSlippageChange={(slippage: number) => {
            setSlippage(slippage);
          }}
          onCurveTypeChange={(curveType: CurveType) => setCurve(curveType)}
          onVersionChange={(version: PoolVersion) => setVersion(version)}
          onSwitch={() => {
            setFromCoin(toCoin);
            setToCoin(fromCoin);
          }}
          onSwap={async (payload) => {
            const { fromCoin, toCoin, interactiveMode } = payload;
            setFromCoin(fromCoin);
            setToCoin(toCoin);
            setInteractiveMode(interactiveMode);
          }}
          onPreview={async () => {}}
          onPropose={async () => {
            // enqueueSnackbar('Propose swap transaction', { variant: 'success' });
          }}
        />
      </Stack>
    </Stack>
  );
}
