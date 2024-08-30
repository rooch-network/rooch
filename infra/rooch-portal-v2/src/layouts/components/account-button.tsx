import type { IconButtonProps } from '@mui/material/IconButton';

import { useWallets, useConnectWallet } from '@roochnetwork/rooch-sdk-kit';

import { Button } from '@mui/material';

export type AccountButtonProps = IconButtonProps & {
  open: boolean;
  photoURL: string;
  displayName: string;
};

export function AccountButton() {
  const wallets = useWallets();
  const { mutateAsync: connectWallet } = useConnectWallet();

  return (
    <Button
      variant="outlined"
      onClick={async () => {
        await connectWallet({ wallet: wallets[0] });
      }}
    >
      Connect Wallet
    </Button>
  );
}
