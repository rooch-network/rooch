'use client';

import type { ReactNode } from 'react';
import type { UserCoin, CurveType, PoolVersion, InteractiveMode } from 'src/components/swap/types';

import { useState, useEffect } from 'react';
import { useCurrentWallet, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import { Stack } from '@mui/material';

import { sleep } from 'src/utils/common';

import Swap from 'src/components/swap/swap';
import { DEFAULT_SLIPPAGE } from 'src/components/swap/types';

const swapCoins = [
  {
    coinType: 'btc',
    decimals: 8,
    symbol: 'BTC',
    name: 'BTC Coin',
    icon: 'https://s2.coinmarketcap.com/static/img/coins/64x64/1.png',
    balance: 0n,
    amount: 0n,
    price: 1,
  },
  {
    coinType: 'rgas',
    decimals: 6,
    symbol: 'RGas',
    name: 'Rooch Gas',
    icon: 'https://s2.coinmarketcap.com/static/img/coins/64x64/1.png',
    balance: 0n,
    amount: 0n,
    price: 1,
  },
];

export default function GasSwapOverview() {
  const [loading, setLoading] = useState<boolean>(false);
  const [coins, setCoins] = useState<UserCoin[]>([]);
  const [fromCoin, setFromCoin] = useState<UserCoin>(swapCoins[0]);
  const [btcBalance, setBtcBalance] = useState(0n);
  const [toCoin, setToCoin] = useState<UserCoin>(swapCoins[1]);
  const [interactiveMode, setInteractiveMode] = useState<InteractiveMode>('from');
  const [slippage, setSlippage] = useState<number>(DEFAULT_SLIPPAGE);
  const [canSelectCurve, setCanSelectCurve] = useState<boolean>(false);
  const [curve, setCurve] = useState<CurveType>('uncorrelated');
  const [warning, setWarning] = useState<ReactNode>();
  const [convertRate, setConvertRate] = useState<number>();
  const [platformFeePercent] = useState<number>(0.003);
  const [platformFeeAmount, setPlatformFeeAmount] = useState<number>();
  const [swapAmount, setSwapAmount] = useState(0n);
  const [slippageAmount, setSlippageAmount] = useState<number>();
  const [canSelectVersion, setCanSelectVersion] = useState<boolean>(false);
  const [version, setVersion] = useState<PoolVersion>(0);

  const address = useCurrentAddress();
  const wallet = useCurrentWallet();

  useEffect(() => {
    async function getBTCBalance() {
      const res = await wallet.wallet?.getBalance();
      console.log('🚀 ~ file: index.tsx:69 ~ getBTCBalance ~ res:', res);
      if (res) {
        setBtcBalance(BigInt(res.confirmed));
      }
    }
    getBTCBalance();
  }, [wallet]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setCoins(swapCoins);
      // setFromCoin(swapCoins[0]);
    }, 500);
    return () => clearTimeout(timer);
  }, []);

  // useEffect(() => {
  //   const execute = async () => {
  //     if (fromCoin && toCoin) {
  //       setLoading(true);
  //       await fetchFee();
  //       // await fetchRate();

  //       setCanSelectVersion(true);
  //       setCurve('uncorrelated');
  //       setCanSelectCurve(true);
  //       setWarning(
  //         'Caution: make sure the pair you are trading should be stable or uncorrelated. i.e USDC/USDT is stable and USDC/BTC is uncorrelated'
  //       );
  //       setLoading(false);
  //     }
  //   };

  //   execute();
  // }, [fromCoin?.coinType, fromCoin?.amount, toCoin?.coinType, toCoin?.amount, interactiveMode]);

  const fetchRate = async (): Promise<void> => {
    if (fromCoin && toCoin) {
      await sleep(500);
    }
  };

  const fetchFee = async (): Promise<void> => {
    await sleep(500);
    setConvertRate(1);
    setPlatformFeeAmount(1);
    setSlippageAmount(1);
    setSwapAmount(1n);
  };

  return (
    <Stack className="w-full justify-center items-center">
      <Stack className="w-3/4 max-w-[600px]">
        <Swap
          loading={loading}
          coins={coins}
          fromCoin={{ ...swapCoins[0], balance: btcBalance }}
          toCoin={toCoin}
          interactiveMode={interactiveMode}
          slippagePercent={slippage}
          canSelectCurve={canSelectCurve}
          curve={curve}
          warning={warning}
          convertRate={convertRate}
          platformFeePercent={platformFeePercent}
          platformFeeAmount={platformFeeAmount}
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
            // setFromCoin(toCoin);
            // setToCoin(fromCoin);
          }}
          onSwap={async (payload) => {
            const { fromCoin, toCoin, interactiveMode } = payload;
            // setFromCoin(fromCoin);
            // setToCoin(toCoin);
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
