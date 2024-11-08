import MarketplaceView from 'src/sections/trade/view';

export const metadata = {
  title: 'MRC20 | Market',
};

export default function Page({ params }: { params: { tick: string } }) {
  const { tick } = params;

  // if (!Object.keys(NETWORK_PACKAGE[NETWORK].tickInfo).includes(tick)) {
  //   return <NotMarketplaceFoundView />;
  // }

  return <MarketplaceView params={params} />;
}
