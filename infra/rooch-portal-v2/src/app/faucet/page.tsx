import WalletGuard from 'src/components/guard/WalletGuard';

import FaucetOverviewView from 'src/sections/faucet/overview';

export const metadata = { title: `Faucet` };

export default function Page() {
  return (
    <WalletGuard>
      <FaucetOverviewView />
    </WalletGuard>
  );
}
