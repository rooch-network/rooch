import type { IconButtonProps } from '@mui/material/IconButton';

import { useState } from 'react';

import { Button } from '@mui/material';

import WalletSelectModal from './wallet-select-modal';

export type AccountButtonProps = IconButtonProps & {
  open: boolean;
  photoURL: string;
  displayName: string;
};

export function AccountButton() {
  const [showWalletSelectModal, setShowWalletSelectModal] = useState(false);

  return (
    <>
      <Button
        variant="outlined"
        onClick={() => {
          setShowWalletSelectModal(true);
        }}
      >
        Connect Wallet
      </Button>
      {showWalletSelectModal && (
        <WalletSelectModal onSelect={() => setShowWalletSelectModal(false)} />
      )}
    </>
  );
}
