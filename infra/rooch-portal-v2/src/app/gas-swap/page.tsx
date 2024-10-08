import WalletGuard from 'src/components/guard/WalletGuard';

import GasSwapOverview from 'src/sections/gas-swap';

export const metadata = { title: `Purchase Gas` };

export default function Page() {
  return (
    <WalletGuard>
      <GasSwapOverview />
    </WalletGuard>
  );
}
