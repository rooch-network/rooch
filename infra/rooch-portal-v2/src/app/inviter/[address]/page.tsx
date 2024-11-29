import WalletGuard from 'src/components/guard/WalletGuard';

import { SettingsView } from 'src/sections/settings/view';

export default function Page({ params }: { params: { address: string } }) {
  // window.localStorage.setItem('inviter', params.address)
  console.log(params)
  return (
    <WalletGuard>
      <SettingsView />
    </WalletGuard>
  );
}



