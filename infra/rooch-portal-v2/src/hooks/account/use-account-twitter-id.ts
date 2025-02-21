import type { ThirdPartyAddress } from '@roochnetwork/rooch-sdk';

import { useQuery } from '@tanstack/react-query';
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit';
import { Args, stringToBytes } from '@roochnetwork/rooch-sdk';

import { useNetworkVariable } from '../use-networks';

export default function useAccountTwitterId(address?: ThirdPartyAddress) {
  const client = useRoochClient();
  const twitterOracleAddress = useNetworkVariable('roochMultiSigAddr');

  const {
    data: twitterId,
    refetch,
    isPending,
    isFetching,
    isError,
  } = useQuery({
    queryKey: ['twitterId', address],
    queryFn: async () => {
      if (!address) {
        return null;
      }
      const res = await client.executeViewFunction({
        address: twitterOracleAddress,
        module: 'twitter_account',
        function: 'resolve_author_id_by_address',
        args: [Args.address(address.toStr())],
      });
      if (res.vm_status === 'Executed') {
        if (res.return_values?.[0].value.value !== '0x00') {
          const _twitterId = (res.return_values?.[0].decoded_value as any).value.vec
            .value[0][0] as string;
          return new TextDecoder('utf-8').decode(
            stringToBytes('hex', _twitterId.replace('0x', ''))
          );
        }
      }
      return null;
    },
    enabled: !!address,
    refetchInterval: 5000,
  });

  return {
    data: twitterId,
    isPending,
    isFetching,
    isError,
    refetch,
  };
}
