import WalletGuard from 'src/components/guard/WalletGuard';

import SwapView from 'src/sections/trade/swap/view';

export const metadata = {
  title: 'Swap',
};

export default function Page() {
  return (
    <WalletGuard>
      <SwapView />
    </WalletGuard>
  );
}
