import { useQuery } from '@tanstack/react-query';
import { useCurrentWallet, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

export default function useAccountBTCBalance() {
  const wallet = useCurrentWallet();
  const address = useCurrentAddress();

  const {
    data: btcBalance,
    isPending,
    isError,
    refetch,
  } = useQuery({
    queryKey: ['btcBalance', address],
    queryFn: async () => {
      if (!wallet.wallet) {
        return 0n;
      }
      const res = await wallet.wallet.getBalance();
      if (res) {
        return BigInt(res.confirmed);
      }
      return 0n;
    },
  });

  return { btcBalance, isPending, isError, refetch };
}
