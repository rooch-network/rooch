export interface TickTradeInfo {
  burn_ratio: string;
  community_ratio: string;
  id: {
    id: string;
  };
  lock_ratio: string;
  tick: string;
  timestamp: string;
  today_volume: string;
  total_volume: string;
  yesterday_volume: string;
}

export default function useMarketTradeData(tick: string) {
  // const { data: trade, isLoading, refetch: refetchTickTradeInfo } = useSuiClientQuery('getObject', {
  //   id: NETWORK_PACKAGE[NETWORK].tickInfo[tick].MARKET_TRADE_INFO,
  //   options: {
  //     showContent: true
  //   }
  // }, {
  //   refetchInterval: 5000
  // })
  return {
    tickTradeInfo: undefined,
    isLoadingTickTradeInfo: false,
    refetchTickTradeInfo: false,
  };
}

export function useBatchMarketTradeData(ticks: string[]) {
  // const { data: trade, isLoading, refetch: refetchTickTradeInfos } = useSuiClientQuery('multiGetObjects', {
  //   ids: ticks.map(i => NETWORK_PACKAGE[NETWORK].tickInfo[i]?.MARKET_TRADE_INFO),
  //   options: {
  //     showContent: true
  //   }
  // }, {
  //   refetchInterval: 5000
  // })

  // const tradeInfo = trade?.map(tradeInfo => (((tradeInfo?.data?.content as any).fields as unknown as any).value)?.fields as TickTradeInfo).filter(i => Boolean(i))

  return {
    tickTradeInfos: [],
    isLoadingTickTradeInfos: false,
    refetchTickTradeInfos: () => {},
  };
}
