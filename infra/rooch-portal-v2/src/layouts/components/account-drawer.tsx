'use client';

import type { IconButtonProps } from '@mui/material/IconButton';

import { useState, useCallback } from 'react';
import { useWallets, useWalletStore, useCurrentAddress } from '@roochnetwork/rooch-sdk-kit';

import Box from '@mui/material/Box';
import { Button } from '@mui/material';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';

import { shortAddress } from 'src/utils/address';

import { varAlpha } from 'src/theme/styles';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { AnimateAvatar } from 'src/components/animate';

import WalletSelectModal from './wallet-select-modal';
import { DisconnectButton } from './disconnect-button';

export type AccountDrawerProps = IconButtonProps & {};

export function AccountDrawer() {
  const theme = useTheme();
  const wallets = useWallets();
  const currentAddress = useCurrentAddress();
  const connectionStatus = useWalletStore((state) => state.connectionStatus);

  const [open, setOpen] = useState(false);

  const [showWalletSelectModal, setShowWalletSelectModal] = useState(false);

  const handleOpenDrawer = useCallback(() => {
    setOpen(true);
  }, []);

  const handleCloseDrawer = useCallback(() => {
    setOpen(false);
  }, []);

  const renderAvatar = (
    <AnimateAvatar
      width={96}
      slotProps={{
        avatar: { src: '/logo/logo.png', alt: 'Rooch' },
        overlay: {
          border: 2,
          spacing: 3,
          color: `linear-gradient(135deg, ${varAlpha(theme.vars.palette.primary.mainChannel, 0)} 25%, ${theme.vars.palette.primary.main} 100%)`,
        },
      }}
    >
      <img src="/logo/logo.png" width="100%" alt="" />
    </AnimateAvatar>
  );

  return (
    <>
      <Button
        variant="outlined"
        onClick={async () => {
          if (connectionStatus === 'connected') {
            handleOpenDrawer();
            return;
          }
          setShowWalletSelectModal(true);
        }}
      >
        {connectionStatus === 'connected'
          ? shortAddress(currentAddress?.toStr(), 8, 6)
          : 'Connect Wallet'}
      </Button>

      {showWalletSelectModal && (
        <WalletSelectModal onSelect={() => setShowWalletSelectModal(false)} />
      )}

      <Drawer
        open={open}
        onClose={handleCloseDrawer}
        anchor="right"
        slotProps={{ backdrop: { invisible: true } }}
        PaperProps={{ sx: { width: 320 } }}
      >
        <IconButton
          onClick={handleCloseDrawer}
          sx={{ top: 12, left: 12, zIndex: 9, position: 'absolute' }}
        >
          <Iconify icon="mingcute:close-line" />
        </IconButton>

        <Scrollbar>
          <Stack alignItems="center" sx={{ pt: 8 }}>
            {renderAvatar}

            <Typography variant="subtitle1" noWrap sx={{ mt: 2 }}>
              {shortAddress(currentAddress?.toStr(), 8, 6)}
            </Typography>

            <Typography variant="body2" sx={{ color: 'text.secondary', mt: 0.5 }} noWrap>
              {shortAddress(currentAddress?.genRoochAddress().toStr(), 8, 6)}
            </Typography>
          </Stack>
        </Scrollbar>

        <Box sx={{ p: 2.5 }}>
          <DisconnectButton onClose={handleCloseDrawer} />
        </Box>
      </Drawer>
    </>
  );
}
