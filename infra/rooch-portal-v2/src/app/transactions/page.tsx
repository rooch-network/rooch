import WalletGuard from 'src/components/guard/WalletGuard';

import TransactionsOverviewView from 'src/sections/transactions/overview';

export const metadata = { title: `Search Transaction` };

export default function Page() {
  return (
    <WalletGuard>
      <TransactionsOverviewView />;
    </WalletGuard>
  );
}
