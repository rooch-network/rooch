'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useCurrentAddress, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { Card, Chip, Stack, CardHeader, Typography, CardContent } from '@mui/material';

import { DashboardContent } from 'src/layouts/dashboard';

import SessionKeysTableCard from './components/session-keys-table-card';

export function SettingsView() {
  const address = useCurrentAddress();
  const router = useRouter();

  const [isAddressLoaded, setIsAddressLoaded] = useState(false);

  const {
    data: sessionKeys,
    isPending: isLoadingSessionKeys,
    refetch: refetchSessionKeys,
  } = useRoochClientQuery(
    'getSessionKeys',
    {
      address: address!,
    },
    { enabled: !!address }
  );

  useEffect(() => {
    if (address !== undefined) {
      setIsAddressLoaded(true);
    }
  }, [address]);

  useEffect(() => {
    if (isAddressLoaded && !address) {
      router.push('/account');
    }
  }, [address, isAddressLoaded, router]);

  return (
    <DashboardContent maxWidth="xl">
      <Card className="mt-4">
        <CardHeader
          title="Rooch Address"
          subheader="Use Rooch address in the application and smart contract development"
        />
        <CardContent className="!pt-2">
          <Stack>
            <Chip className="justify-start w-fit" label={address?.genRoochAddress().toStr()} />
            <Typography className="!mt-2 text-gray-400 !text-sm">
              This is your Rooch Address mapping from the wallet address
            </Typography>
          </Stack>
        </CardContent>
      </Card>
      <SessionKeysTableCard
        sessionKeys={sessionKeys}
        isPending={isLoadingSessionKeys}
        refetchSessionKeys={refetchSessionKeys}
        address={address?.toStr() || ''}
      />
    </DashboardContent>
  );
}
