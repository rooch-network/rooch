'use client';

import type { ReactNode} from 'react';
import type { CurveType, PoolVersion, InteractiveMode } from 'src/components/swap/types';

import BigNumber from 'bignumber.js';
import { useState, useEffect } from 'react';
import { Args } from '@roochnetwork/rooch-sdk';
import { useRoochClient, useCurrentWallet, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit'

import { Stack } from '@mui/material';

import Swap from 'src/components/swap/swap';
import { toast } from 'src/components/snackbar';

import { useNetworkVariable } from '../../hooks/use-networks'
import WalletSwitchNetworkModal from '../../layouts/components/wallet-switch-network-modal'

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
  const btcGasAddress = useNetworkVariable("btcGasAddress")
  const gasMarketAddress = useNetworkVariable("gasMarketAddress")
  const [loading, setLoading] = useState<boolean>(false);
  const [submitting, setSubmitting] = useState<boolean>(false);
  const [btcBalance, setBtcBalance] = useState(0n);
  const [rGasBalance, setRGasBalance] = useState(0n);
  const [interactiveMode, setInteractiveMode] = useState<InteractiveMode>('from');
  const [curve, setCurve] = useState<CurveType>('uncorrelated');
  const [warning, setWarning] = useState<ReactNode>();
  const [convertRate, setConvertRate] = useState<number>();
  const [platformFeePercent] = useState<number>(0.003);
  const [version, setVersion] = useState<PoolVersion>(0);
  const [networkValid, setNetworkValid] = useState<boolean>(true);

  const [fromSwapAmount, setFromSwapAmount] = useState(0n);
  const [toSwapAmount, setToSwapAmount] = useState(0n);
  const [txHash, setTxHash] = useState<string>();

  const address = useCurrentAddress();
  const wallet = useCurrentWallet();
  const client = useRoochClient();

  useEffect(() => {
    async function getBTCBalance() {
      const res = await wallet.wallet?.getBalance();
      if (res) {
        setBtcBalance(BigInt(res.confirmed));
      }
    }
    async function getRGasBalance() {
      if (!address) {
        return;
      }
      const res = await client.getBalance({
        owner: address?.genRoochAddress().toStr(),
        coinType: '0x3::gas_coin::RGas',
      });
      if (res) {
        setRGasBalance(BigInt(res.balance));
      }
    }
    getBTCBalance();
    getRGasBalance();
  }, [wallet, address, client]);
  console.log(btcBalance)

  useEffect(() => {
    async function fetchRate() {
      try {
        setLoading(true);
        const res = await client.executeViewFunction({
          address: gasMarketAddress,
          module: 'gas_market',
          function: 'btc_to_rgas',
          args: [Args.u64(fromSwapAmount)],
        });
        setToSwapAmount(BigInt(Number(res.return_values?.[0]?.decoded_value || 0)) || 0n);
        setConvertRate(
          new BigNumber(Number(res.return_values?.[0]?.decoded_value || 0))
            .div(fromSwapAmount.toString())
            .toNumber()
        );
      } catch (error) {
        toast.error(String(error));
      } finally {
        setLoading(false);
      }
    }
    fetchRate();
  }, [client, fromSwapAmount, gasMarketAddress]);

  console.log(networkValid)
  return (
    <Stack className="w-full justify-center items-center">
      <Stack className="w-3/4 max-w-[600px]">
        <WalletSwitchNetworkModal onChecked={(isValid) => setNetworkValid(isValid)}/>
        <Swap
          isValid={networkValid}
          hiddenValue
          fixedSwap
          loading={loading}
          coins={[]}
          fromCoin={{ ...swapCoins[0], balance: btcBalance, amount: fromSwapAmount }}
          toCoin={{ ...swapCoins[1], balance: rGasBalance, amount: toSwapAmount }}
          interactiveMode={interactiveMode}
          canSelectCurve={false}
          curve={curve}
          txHash={txHash}
          warning={warning}
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
                toAddress: btcGasAddress,
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
