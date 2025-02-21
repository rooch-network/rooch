'use client';

import type { CurveType, PoolVersion, InteractiveMode } from 'src/components/gas-swap/types';

import { useState } from 'react';
import { useCurrentWallet, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import { Stack } from '@mui/material';

import { useNetworkVariable } from 'src/hooks/use-networks';
import useGasMarketRate from 'src/hooks/gas/use-gas-market-rate';
import useAccountBTCBalance from 'src/hooks/account/use-account-btc-balance';
import useAccountRGasBalance from 'src/hooks/account/use-account-rgas-balance';

import WalletSwitchNetworkModal from 'src/layouts/components/wallet-switch-network-modal';

import Swap from 'src/components/gas-swap/swap';
import { toast } from 'src/components/snackbar';

const swapCoins = [
  {
    coinType: 'btc',
    decimals: 8,
    symbol: 'BTC',
    name: 'BTC Coin',
    icon: 'https://s2.coinmarketcap.com/static/img/coins/64x64/1.png',
    price: 0,
  },
  {
    coinType: 'rgas',
    decimals: 8,
    symbol: 'RGas',
    name: 'Rooch Gas',
    icon: '/logo/logo-square.svg',
    price: 0,
  },
];

export default function GasSwapOverview() {
  const gasMarketCfg = useNetworkVariable('gasMarket');

  const [submitting, setSubmitting] = useState<boolean>(false);
  const [interactiveMode, setInteractiveMode] = useState<InteractiveMode>('from');
  const [curve, setCurve] = useState<CurveType>('uncorrelated');
  const [platformFeePercent] = useState<number>(0.003);
  const [version, setVersion] = useState<PoolVersion>(0);
  const [networkValid, setNetworkValid] = useState<boolean>(true);

  const [fromSwapAmount, setFromSwapAmount] = useState(0n);
  const [txHash, setTxHash] = useState<string>();

  const address = useCurrentAddress();
  const wallet = useCurrentWallet();

  const { btcBalance, isPending: isBTCBalancePending } = useAccountBTCBalance();
  const { data: rGasBalance, isPending: isRGasBalancePending } = useAccountRGasBalance(
    address?.genRoochAddress().toStr()
  );
  const {
    toSwapAmount,
    convertRate,
    isPending: isGasMarketRatePending,
  } = useGasMarketRate(fromSwapAmount);

  return (
    <Stack className="w-full justify-center items-center">
      <Stack className="w-3/4 max-w-[600px]">
        <WalletSwitchNetworkModal onChecked={(isValid) => setNetworkValid(isValid)} />
        <Swap
          isValid={networkValid}
          hiddenValue
          fixedSwap
          loading={isGasMarketRatePending || isBTCBalancePending || isRGasBalancePending}
          coins={[]}
          fromCoin={{ ...swapCoins[0], balance: btcBalance || 0n, amount: fromSwapAmount }}
          toCoin={{
            ...swapCoins[1],
            balance: rGasBalance?.balance || 0n,
            amount: toSwapAmount || 0n,
          }}
          interactiveMode={interactiveMode}
          canSelectCurve={false}
          curve={curve}
          txHash={txHash}
          convertRate={convertRate}
          platformFeePercent={platformFeePercent}
          priceImpact={0}
          priceImpactSeverity="normal"
          proposing={submitting}
          version={version}
          onSlippageChange={(slippage: number) => {}}
          onCurveTypeChange={(curveType: CurveType) => setCurve(curveType)}
          onVersionChange={(version: PoolVersion) => setVersion(version)}
          onSwap={async (payload) => {
            const { fromCoin, toCoin, interactiveMode } = payload;
            if (!fromCoin || !toCoin) {
              return;
            }
            setFromSwapAmount(fromCoin.amount);
            setInteractiveMode(interactiveMode);
          }}
          onPreview={async () => {
            try {
              setSubmitting(true);
              const txHash = await wallet.wallet?.sendBtc({
                toAddress: gasMarketCfg.recipientBTCAddress,
                satoshis: Number(fromSwapAmount.toString()),
              });
              setTxHash(txHash);
            } catch (error) {
              toast.error(String(error.message));
            } finally {
              setSubmitting(false);
            }
          }}
          onPropose={async () => {}}
        />
      </Stack>
    </Stack>
  );
}
