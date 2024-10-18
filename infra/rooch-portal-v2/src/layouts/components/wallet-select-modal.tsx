import { useWallets } from '@roochnetwork/rooch-sdk-kit';

import { Stack, Dialog, DialogTitle, DialogContent } from '@mui/material';

import WalletButton from './wallet-button';
import { isMainNetwork } from '../../utils/env'

export default function WalletSelectModal({ onSelect }: { onSelect: () => void }) {
  const isMain = isMainNetwork()
  const wallets = useWallets().filter((item) => isMain || !isMain && item.getName() === 'UniSat');

  return (
    <Dialog
      open
      onClose={() => {
        onSelect();
      }}
    >
      <DialogTitle sx={{ pb: 2 }}>Select wallet</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          overflow: 'unset',
        }}
      >
        <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1, pb: 4 }}>
          {wallets.slice(0, 3).map((w) => (
            <WalletButton key={w.getName()} wallet={w} onSelect={onSelect} />
          ))}
        </Stack>
      </DialogContent>
    </Dialog>
  );
}
