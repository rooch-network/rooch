'use client';

import { useState, useEffect } from 'react';
import { isValidBitcoinAddress } from '@roochnetwork/rooch-sdk';
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

import { LoadingButton } from '@mui/lab';
import { Box, Card, Chip, Stack, CardHeader, CardContent } from '@mui/material';

import { useRouter } from 'src/routes/hooks';

import { useNetworkVariable } from 'src/hooks/use-networks'

import { BitcoinAddressToRoochAddress } from 'src/utils/address';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';

import { formatCoin } from '../../utils/format-number'

export function FaucetView({ address }: { address: string }) {
  const [viewAddress, setViewAddress] = useState<string>();
  const [viewRoochAddress, setViewRoochAddress] = useState<string>();
  const [faucetStatus, setFaucetStatus] = useState<boolean>(false);
  const faucetUrl = useNetworkVariable("faucetUrl")
  const [errorMsg, setErrorMsg] = useState<string>();

  const router = useRouter();

  useEffect(() => {
    if (isValidBitcoinAddress(address)) {
      setViewAddress(address);
      try {
        setViewRoochAddress(BitcoinAddressToRoochAddress(address!).toHexAddress());
      } catch (error) {
        toast.error('Invalid query address');
        router.push('/search');
      }
    } else {
      toast.error('Invalid query address');
      router.push('/search');
    }
  }, [address, router]);

  const {
      data,
      isPending,
      refetch,
    } = useRoochClientQuery(
      'getBalance',
      {
        owner: viewRoochAddress!,
        coinType: '0x3::gas_coin::RGas'
      },
      { refetchInterval: 5000 }
    );

  const fetchFaucet = async ()=> {
    setFaucetStatus(true)
    try {
      const payload = JSON.stringify({
        claimer: viewAddress
      });
      const response = await fetch(faucetUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: payload,
      });

      if (!response.ok) {
        const data = await response.json()
        console.log(data)
        if (response.status === 500 && data.error.includes('UTXO value is zero')) {
          const msg = 'Claim failed, Not found UTXO'
          setErrorMsg(msg)
          toast.error(msg)
          return
        }

        toast.error('Network response was not ok');
        return
      }

      const d = await response.json();
      await refetch()
      toast.success(`Faucet Success! RGas: ${d.gas}`);
    } catch (error) {
      console.error('Error:', error);
      toast.error(`faucet error: ${error}`);
    } finally {
      setFaucetStatus(false)
    }
  }

  if (!viewAddress) {
    return null;
  }

  return (
    <DashboardContent maxWidth="xl">
      <Card>
        <CardHeader title="Gas Faucet" sx={{ mb: 1 }} />
        <CardContent className="!pt-0">
          <Stack spacing={2}>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip
                className="w-fit"
                label='Claim Address:'
                variant="soft"
                color="default"
              />
              <Box className="text-gray-400 text-sm font-medium">({viewAddress})</Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip
                className="w-fit"
                label='Claim Rooch Address:'
                variant="soft"
                color="default"
              />
              <Box className="text-gray-400 text-sm font-medium">({viewRoochAddress})</Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip
                className="w-fit"
                label='RGas Balance:'
                variant="soft"
                color="secondary"
              />
              <Chip
                className="w-fit"
                label={formatCoin(Number(data?.balance || 0), data?.decimals || 0, 2)}
                variant="soft"
                color="default"
              />
            </Stack>
            <LoadingButton
              variant='soft'
              color='primary'
              disabled={errorMsg !== undefined}
              loading={isPending || faucetStatus}
              onClick={fetchFaucet}
            >
              {errorMsg || 'Claim'}
            </LoadingButton>
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
