import MarketPlaceHistoryView from 'src/sections/history/view';

// ----------------------------------------------------------------------

export const metadata = {
  title: 'Market | History',
};

export default function Page({ params }: { params: { tick: string } }) {
  return <MarketPlaceHistoryView tick={params.tick} />;
}
