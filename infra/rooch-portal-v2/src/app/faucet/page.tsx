import WalletGuard from 'src/components/guard/WalletGuard';

import FaucetOverviewView from 'src/sections/faucet/overview';

export const metadata = { title: `My Account` };

export default function Page() {
  return (
    <WalletGuard>
      <FaucetOverviewView />;
    </WalletGuard>
  );
}
