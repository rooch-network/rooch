import MarketplaceView from 'src/sections/trade/view';

export const metadata = {
  title: 'Market | Orderbook',
};

export default function Page({ params }: { params: { tick: string } }) {
  console.log('🚀 ~ file: page.tsx:8 ~ Page ~ params:', params);
  // if (!Object.keys(NETWORK_PACKAGE[NETWORK].tickInfo).includes(tick)) {
  //   return <NotMarketplaceFoundView />;
  // }

  return <MarketplaceView params={params} />;
}
