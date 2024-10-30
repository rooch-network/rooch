'use client';

import { useState, useEffect } from 'react';
import { Args, isValidBitcoinAddress } from '@roochnetwork/rooch-sdk';
import { useRoochClient, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Box, Card, Chip, Stack, CardHeader, CardContent } from '@mui/material';

import { useRouter } from 'src/routes/hooks';
import useAddressChanged from 'src/routes/hooks/useAddressChanged';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatCoin } from 'src/utils/format-number';
import { BitcoinAddressToRoochAddress } from 'src/utils/address';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';

const ERROR_MSG: Record<string, string> = {
  1: 'Faucet Not Open',
  2: 'Invalid UTXO',
  3: 'Faucet Not Enough RGas',
  4: 'Already Claimed',
  5: 'UTXO Value Is Zero',
};

export function FaucetView({ address }: { address: string }) {
  const [viewAddress, setViewAddress] = useState<string>();
  const [viewRoochAddress, setViewRoochAddress] = useState<string>();
  const [faucetStatus, setFaucetStatus] = useState<boolean>(false);
  const faucetUrl = useNetworkVariable('faucetUrl');
  const [errorMsg, setErrorMsg] = useState<string>();
  const client = useRoochClient();
  const faucetAddress = useNetworkVariable('faucetAddress');
  const faucetObject = useNetworkVariable('faucetObject');
  const [claimGas, setClaimGas] = useState(0);
  const router = useRouter();

  useAddressChanged({ address, path: 'faucet' });

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

  const { data, isPending, refetch } = useRoochClientQuery(
    'getBalance',
    {
      owner: viewRoochAddress!,
      coinType: '0x3::gas_coin::RGas',
    },
    { refetchInterval: 5000 }
  );

  useEffect(() => {
    if (!viewRoochAddress) {
      return;
    }
    setFaucetStatus(true);
    client
      .queryUTXO({
        filter: {
          owner: address,
        },
      })
      .then(async (result) => {
        const utxoIds = result.data.map((item) => item.id);
        if (utxoIds) {
          const result = await client.executeViewFunction({
            target: `${faucetAddress}::gas_faucet::check_claim`,
            args: [
              Args.objectId(faucetObject),
              Args.address(viewRoochAddress),
              Args.vec('objectId', utxoIds),
            ],
          });

          if (result.vm_status === 'Executed') {
            const gas = Number(formatCoin(Number(result.return_values![0].decoded_value), 8, 2));
            setClaimGas(gas);
          } else if ('MoveAbort' in result.vm_status) {
            setErrorMsg(ERROR_MSG[Number(result.vm_status.MoveAbort.abort_code)]);
          }
        } else {
          setErrorMsg('Not found utxo');
        }
      })
      .finally(() => {
        setFaucetStatus(false);
      });
  }, [address, client, faucetAddress, faucetObject, viewRoochAddress]);

  const fetchFaucet = async () => {
    setFaucetStatus(true);
    try {
      const payload = JSON.stringify({
        claimer: viewAddress,
      });
      const response = await fetch(faucetUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: payload,
      });

      if (!response.ok) {
        const data = await response.json();
        console.log(data);
        if (response.status === 500 && data.error.includes('UTXO value is zero')) {
          const msg = 'Claim failed, Not found UTXO';
          setErrorMsg(msg);
          toast.error(msg);
          return;
        }

        toast.error('Network response was not ok');
        return;
      }

      const d = await response.json();
      await refetch();
      toast.success(
        `Faucet Success! RGas: ${formatCoin(Number(d.gas || 0), data?.decimals || 0, 2)}`
      );
    } catch (error) {
      console.error('Error:', error);
      toast.error(`faucet error: ${error}`);
    } finally {
      setFaucetStatus(false);
    }
  };

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
              <Chip className="w-fit" label="Claim Address:" variant="soft" color="default" />
              <Box className="text-gray-400 text-sm font-medium">({viewAddress})</Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip className="w-fit" label="Claim Rooch Address:" variant="soft" color="default" />
              <Box className="text-gray-400 text-sm font-medium">({viewRoochAddress})</Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip className="w-fit" label="RGas Balance:" variant="soft" color="secondary" />
              <Box className="text-gray-400 text-sm font-medium">
                {formatCoin(Number(data?.balance || 0), data?.decimals || 0, 2)}
              </Box>
            </Stack>
            {errorMsg
              ? 'You cannot claim gas, Please make sure the current address has a valid utxo and try again'
              : ''}
            <LoadingButton
              variant="soft"
              color="primary"
              disabled={errorMsg !== undefined}
              loading={isPending || faucetStatus}
              onClick={fetchFaucet}
            >
              {errorMsg || `Claim: ${claimGas} RGas`}
            </LoadingButton>
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
