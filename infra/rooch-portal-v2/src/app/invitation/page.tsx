
import WalletGuard from 'src/components/guard/WalletGuard';
import InvitationOverviewView from 'src/sections/invitations/overview';

export const metadata = { title: `Invitation` };

export default function Page() {
  return (<WalletGuard>
    <InvitationOverviewView />
  </WalletGuard>);
}
