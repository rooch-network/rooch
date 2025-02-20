import BigNumber from 'bignumber.js';
import { Args } from '@roochnetwork/rooch-sdk';
import { useQuery } from '@tanstack/react-query';
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit';

import { useNetworkVariable } from '../use-networks';

export default function useGasMarketRate(fromSwapAmount: bigint) {
  const gasMarketCfg = useNetworkVariable('gasMarket');
  const client = useRoochClient();

  const {
    data: gasMarketRate,
    isPending,
    isError,
    refetch,
  } = useQuery({
    queryKey: ['gasMarketRate', fromSwapAmount.toString()],
    queryFn: async () => {
      const res = await client.executeViewFunction({
        address: gasMarketCfg.address,
        module: 'gas_market',
        function: 'btc_to_rgas',
        args: [Args.u64(fromSwapAmount)],
      });
      return {
        toSwapAmount: BigInt(Number(res.return_values?.[0]?.decoded_value || 0)),
        convertRate: new BigNumber(Number(res.return_values?.[0]?.decoded_value || 0))
          .div(fromSwapAmount.toString())
          .toNumber(),
      };
    },
    enabled: !!fromSwapAmount && Number(fromSwapAmount) > 0,
  });

  return {
    toSwapAmount: gasMarketRate?.toSwapAmount,
    convertRate: gasMarketRate?.convertRate,
    isPending,
    isError,
    refetch,
  };
}
