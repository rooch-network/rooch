import WalletGuard from 'src/components/guard/WalletGuard';

import AccountOverviewView from 'src/sections/account/overview';

export const metadata = { title: `My Account` };

export default function Page() {
  return (
    <WalletGuard>
      <AccountOverviewView />
    </WalletGuard>
  );
}
