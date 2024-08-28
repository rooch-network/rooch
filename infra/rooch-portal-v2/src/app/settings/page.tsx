import WalletGuard from 'src/components/guard/WalletGuard';

import { SettingsView } from 'src/sections/settings/view';

export const metadata = { title: `Settings` };

export default function Page() {
  return (
    <WalletGuard>
      <SettingsView />
    </WalletGuard>
  );
}
