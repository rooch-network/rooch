import { useQuery } from '@tanstack/react-query';
import { getMarketListPagination } from './use-market-data';
import { useMemo } from 'react';
import BigNumber from 'bignumber.js';
import { SUI_DECIMALS } from 'src/config/trade';

export default function useFloorPrice(tick: string) {
  const { data } = useQuery({
    queryKey: ['floor-price'],
    queryFn: async () => {
      return await getMarketListPagination(0, 0, tick);
    },
    enabled: !!tick,
    refetchInterval: 10 * 1000,
    staleTime: 10 * 10000,
  });
  console.log('🚀 ~ file: useFloorPrice.tsx:16 ~ useFloorPrice ~ data:', data);

  const floorPrice = useMemo(() => {
    return data && data[0]
      ? new BigNumber(data[0].price)
          .div(data[0].amt)
          .div(new BigNumber(10).pow(SUI_DECIMALS))
          .toNumber()
      : undefined;
  }, [data]);

  return floorPrice;
}
