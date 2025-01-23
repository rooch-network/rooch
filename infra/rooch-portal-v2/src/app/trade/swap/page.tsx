import WalletGuard from 'src/components/guard/WalletGuard';

import PoolView from 'src/sections/trade/swap/view';

export const metadata = {
  title: 'swap',
};

export default function Page({ params }: { params: { tick: string } }) {
  return (
    <WalletGuard>
      <PoolView />
    </WalletGuard>
  );
}
