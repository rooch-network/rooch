import WalletGuard from 'src/components/guard/WalletGuard';

import AssetsOverviewView from 'src/sections/assets/overview';

export const metadata = { title: `Search Account Assets` };

export default function Page() {
  return (
    <WalletGuard>
      <AssetsOverviewView />
    </WalletGuard>
  );
}
