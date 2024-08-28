import type { ButtonProps } from '@mui/material/Button';
import type { Theme, SxProps } from '@mui/material/styles';

import { useCallback } from 'react';
import { useWalletStore } from '@roochnetwork/rooch-sdk-kit';

import Button from '@mui/material/Button';

import { useRouter } from 'src/routes/hooks';

type Props = ButtonProps & {
  sx?: SxProps<Theme>;
  onClose?: () => void;
};

export function DisconnectButton({ onClose, ...other }: Props) {
  const router = useRouter();

  const connectionStatus = useWalletStore((state) => state.connectionStatus);
  const setWalletDisconnected = useWalletStore((state) => state.setWalletDisconnected);

  const handleLogout = useCallback(async () => {
    try {
      if (connectionStatus === 'connected') {
        setWalletDisconnected();
      }
      onClose?.();
      router.refresh();
    } catch (error) {
      console.error(error);
    }
  }, [connectionStatus, onClose, router, setWalletDisconnected]);

  return (
    <Button fullWidth variant="soft" size="large" color="error" onClick={handleLogout} {...other}>
      Disconnect
    </Button>
  );
}
