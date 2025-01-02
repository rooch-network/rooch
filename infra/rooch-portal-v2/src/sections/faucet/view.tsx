'use client';

import { useState, useEffect, useCallback } from "react";
import { CopyToClipboard } from 'react-copy-to-clipboard'
import { Args, isValidBitcoinAddress } from '@roochnetwork/rooch-sdk';
import { useRoochClient, useCurrentNetwork, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Box, Card, Chip, Stack, CardHeader, CardContent } from '@mui/material';

import { useRouter } from 'src/routes/hooks';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatCoin } from 'src/utils/format-number';
import { BitcoinAddressToRoochAddress } from 'src/utils/address';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';

import { paths } from '../../routes/paths'
import { INVITER_ADDRESS_KEY } from "../../utils/inviter"

const FAUCET_NOT_OPEN= 'Faucet Not Open'
const INVALID_UTXO = 'Invalid UTXO'
const FAUCET_NOT_ENOUGH_RGAS = 'Faucet Not enough RGas'
const ALREADY_CLAIMED = 'Already Claimed'
const UTXO_VALUE_IS_ZERO = 'UTXO Value Is Zero'

const ERROR_MSG: Record<string, string> = {
  1: FAUCET_NOT_OPEN,
  2: INVALID_UTXO,
  3: FAUCET_NOT_ENOUGH_RGAS,
  4: ALREADY_CLAIMED,
  5: UTXO_VALUE_IS_ZERO,
};

export function FaucetView({ address }: { address: string }) {
  const [viewAddress, setViewAddress] = useState<string>();
  const [viewRoochAddress, setViewRoochAddress] = useState<string>();
  const [faucetStatus, setFaucetStatus] = useState<boolean>(false);
  const faucet = useNetworkVariable('faucet');
  const [errorMsg, setErrorMsg] = useState<string>();
  const client = useRoochClient();
  const [claimGas, setClaimGas] = useState(0);
  const router = useRouter();
  const network = useCurrentNetwork();
  const [needCheck, setNeedCheck] = useState(false);

  useEffect(() => {
    const inviterAddress = window.localStorage.getItem(INVITER_ADDRESS_KEY)
    if (inviterAddress && inviterAddress.length > 0) {
      router.push(`/faucet/inviter/${inviterAddress}`)
    }
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

  const checkClaim = useCallback(() => {
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
            target: `${faucet.address}::gas_faucet::check_claim`,
            args: [
              Args.objectId(faucet.obj),
              Args.address(viewRoochAddress),
              Args.vec('objectId', utxoIds),
            ],
          });

          if (result.vm_status === 'Executed') {
            const gas = Number(formatCoin(Number(result.return_values![0].decoded_value), 8, 2));
            setClaimGas(gas);
            setNeedCheck(false);
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
  }, [address, client, faucet, viewRoochAddress])

  useEffect(() => {
    checkClaim()
  }, [checkClaim]);

  const fetchFaucet = async () => {

    if (errorMsg === ALREADY_CLAIMED) {
      router.push(paths.dashboard['gas-swap'])
      return
    }

    setFaucetStatus(true);
    try {
      const payload = JSON.stringify({
        claimer: viewAddress,
      });
      const response = await fetch(`${faucet.url}/faucet`, {
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
      setNeedCheck(true);
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
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={0.5} sx={{ width: '100%' }}>
      <CardHeader title="Gas Faucet" sx={{ mb: 1, flex: 1 }} />
      <Box sx={{mr:1.5}}>
        <CopyToClipboard onCopy={() => {
          toast.success('Copy to you clipboard')
        }} text={`https://${network === 'mainnet' ? '':'test-'}portal.rooch.network/inviter/${viewAddress}`}>
              <Chip className="justify-start w-fit" label="Share" />
            </CopyToClipboard>
      </Box>
    </Stack>
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
              ? errorMsg === ALREADY_CLAIMED ? 'You Already Claimed RGAS'
              : 'You cannot claim gas, Please make sure the current address has a valid utxo and try again' : ''}
            <LoadingButton
              variant="soft"
              color="primary"
              disabled={errorMsg !== undefined && errorMsg !== ALREADY_CLAIMED}
              loading={isPending || faucetStatus}
              onClick={needCheck ? checkClaim : fetchFaucet}
            >
              {errorMsg === ALREADY_CLAIMED ? 'Purchase RGas' : errorMsg || needCheck ? 'Check' : `Claim: ${claimGas} RGas`}
            </LoadingButton>
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
