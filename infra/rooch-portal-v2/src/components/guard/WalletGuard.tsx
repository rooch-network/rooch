'use client';

import type { ReactNode } from 'react';

import { useWalletStore } from '@roochnetwork/rooch-sdk-kit';

import { Box, Card, Stack, CardHeader, CardContent } from '@mui/material';

import { DashboardContent } from 'src/layouts/dashboard';
import { AccountDrawer } from 'src/layouts/components/account-drawer';

import { Iconify } from '../iconify';

export default function WalletGuard({ children }: { children: ReactNode }) {
  const connectionStatus = useWalletStore((state) => state.connectionStatus);

  if (connectionStatus === 'connected') {
    return children;
  }

  return (
    <DashboardContent maxWidth="xl">
      <Card>
        <CardHeader
          title={
            <Stack
              direction="row"
              className="w-full"
              alignItems="center"
              justifyContent="center"
              spacing={1}
            >
              <Box>Wallet Required</Box>
              <Iconify icon="solar:lock-keyhole-line-duotone" width="20px" />
            </Stack>
          }
          titleTypographyProps={{ sx: { textAlign: 'center' } }}
          subheader="Please connect your wallet to access account info"
          subheaderTypographyProps={{ sx: { textAlign: 'center', mt: 1 } }}
          sx={{ mb: 2 }}
        />
        <CardContent>
          <Stack
            direction="column"
            justifyContent="center"
            alignItems="center"
            className="w-full"
            spacing={2}
          >
            <Iconify icon="solar:wallet-money-bold-duotone" width="64px" />
            <AccountDrawer />
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
