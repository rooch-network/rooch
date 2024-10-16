import type { WalletNetworkType } from '@roochnetwork/rooch-sdk-kit';

import { useState, useEffect } from 'react'
import { useCurrentWallet, useCurrentNetwork } from '@roochnetwork/rooch-sdk-kit'

import { LoadingButton } from '@mui/lab'
import { Stack, Dialog, DialogTitle, DialogContent } from '@mui/material'

export default function WalletSwitchNetworkModal() {
  const roochNetwork = useCurrentNetwork();
  const wallet = useCurrentWallet();

  const [showWalletSelectNetworkModal, setShowWalletSelectNetworkModal] = useState<boolean>(false);
  // rooch testnet needs it after opening other wallets
  const [targetNetwork, setTargetNetwork] = useState<WalletNetworkType | undefined>(undefined);


  useEffect(() => {
    const checkEnv = async () => {
      const walletNetwork = await wallet.wallet?.getNetwork()
      if (roochNetwork === 'testnet') {
        if (walletNetwork !== 'testnet') {
          setShowWalletSelectNetworkModal(true)
          setTargetNetwork('testnet')
        }
      } else if (roochNetwork === 'mainnet') {
        if (walletNetwork !== 'livenet') {
          setShowWalletSelectNetworkModal(true)
          setTargetNetwork('livenet')
        }
      }
    }

    checkEnv()
  }, [roochNetwork, wallet.wallet])
  return (
    showWalletSelectNetworkModal ? <Dialog
      open
      onClose={()=> setShowWalletSelectNetworkModal(false)}
    >
      <DialogTitle sx={{ pb: 2 }}>Switch Network</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          overflow: 'unset',
        }}
      >
        <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1, pb: 4 }}>
          {`The current wallet network is not ${targetNetwork}`}
          <LoadingButton
            variant='soft'
            color='primary'
            onClick={() => {wallet?.wallet?.switchNetwork(targetNetwork!)}}
          >
            {`Switch to ${targetNetwork}`}
          </LoadingButton>
        </Stack>
      </DialogContent>
    </Dialog>:<></>
  );
}
