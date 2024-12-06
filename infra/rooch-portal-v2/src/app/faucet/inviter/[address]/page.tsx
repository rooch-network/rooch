import WalletGuard from 'src/components/guard/WalletGuard';

import { InviterFaucetView } from 'src/sections/faucet/inviter';

export const metadata = { title: `Faucet` };

export default function Page({ params }: { params: { address: string } }) {
  return <WalletGuard>
    <InviterFaucetView inviterAddress={params.address} />
  </WalletGuard>
}
