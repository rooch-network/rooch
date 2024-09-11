import type { Wallet } from '@roochnetwork/rooch-sdk-kit';

import { useState, useEffect } from 'react';
import { useConnectWallet } from '@roochnetwork/rooch-sdk-kit';

import { Stack, Button, CircularProgress } from '@mui/material';

export default function WalletButton({
  wallet,
  onSelect,
}: {
  wallet: Wallet;
  onSelect: () => void;
}) {
  const { mutateAsync: connectWallet } = useConnectWallet();

  const [walletInstalled, setWalletInstalled] = useState(false);
  const [checkingInstall, setCheckingInstall] = useState(false);

  useEffect(() => {
    async function checkWalletInstalled() {
      setCheckingInstall(true);
      const installed = await wallet.checkInstalled();
      setWalletInstalled(installed);
      setCheckingInstall(false);
    }
    checkWalletInstalled();
  }, [wallet]);

  return (
    <Button
      disabled={walletInstalled === false || checkingInstall}
      onClick={async () => {
        await connectWallet({ wallet });
        onSelect();
      }}
    >
      <Stack direction="row" spacing={1.5} alignItems="center">
        <Stack>
          <img src={wallet.getIcon()} width="36px" alt="" />
        </Stack>
        <Stack>{wallet.getName()}</Stack>
        <Stack>{checkingInstall && <CircularProgress size={24} color="info" />}</Stack>
      </Stack>
    </Button>
  );
}
