import type { Wallet } from '@roochnetwork/rooch-sdk-kit';

import { useState, useEffect } from 'react';
import { useConnectWallet } from '@roochnetwork/rooch-sdk-kit';

import { Stack, Button, CircularProgress } from '@mui/material';

import { toast } from 'src/components/snackbar'

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
        try {
          await connectWallet({ wallet });
        } catch (e) {
          if (wallet.getName() === 'OneKey' && e.message.includes('Invalid address')) {
            toast.error('Please disconnect and re-authorize the taproot address')
          }
        }
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
