import { useQuery } from '@tanstack/react-query';
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit';

import { GAS_COIN_TYPE } from 'src/config/constant';

export default function useAccountRGasBalance(address?: string) {
  const client = useRoochClient();

  const {
    data: rGasBalance,
    refetch,
    isPending,
    isError,
  } = useQuery({
    queryKey: ['RGasBalance', address],
    queryFn: async () => {
      if (!address) {
        return null;
      }
      const res = await client.getBalance({
        owner: address,
        coinType: GAS_COIN_TYPE,
      });
      return res;
    },
    enabled: !!address,
    refetchInterval: 5000,
  });

  return {
    data: {
      ...rGasBalance,
      balance: BigInt(rGasBalance?.balance || 0),
    },
    isPending,
    isError,
    refetch,
  };
}
