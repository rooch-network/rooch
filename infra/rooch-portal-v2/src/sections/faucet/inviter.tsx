'use client';

import { useState, useEffect } from 'react';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import {
  useCurrentAddress, useCurrentSession,
  useRoochClient,
  useRoochClientQuery,
} from "@roochnetwork/rooch-sdk-kit";

import { LoadingButton } from '@mui/lab';
import { Box, Card, Chip, Stack, CardHeader, CardContent } from '@mui/material';

import { useRouter } from 'src/routes/hooks';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatCoin } from 'src/utils/format-number';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';

import { paths } from '../../routes/paths';
import { INVITER_ADDRESS_KEY } from '../../utils/inviter';
import SessionKeyGuardButton from "../../components/auth/session-key-guard-button";

const FAUCET_NOT_OPEN = 'Faucet Not Open';
const INVALID_UTXO = 'Invalid UTXO';
const FAUCET_NOT_ENOUGH_RGAS = 'Faucet Not enough RGas';
const ALREADY_CLAIMED = 'Already Claimed';
const UTXO_VALUE_IS_ZERO = 'UTXO Value Is Zero';

const ERROR_MSG: Record<string, string> = {
  1: FAUCET_NOT_OPEN,
  2: INVALID_UTXO,
  3: FAUCET_NOT_ENOUGH_RGAS,
  4: ALREADY_CLAIMED,
  5: UTXO_VALUE_IS_ZERO,
};

export function InviterFaucetView({ inviterAddress }: { inviterAddress: string }) {
  const router = useRouter();

  const client = useRoochClient();
  const faucetAddress = useNetworkVariable('faucetAddress');
  const faucetObject = useNetworkVariable('faucetObject');
  const inviterCA = '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977';
  const inviterConf = `${inviterCA}::invitation::InvitationConf`;
  const session = useCurrentSession()

  const viewAddress = useCurrentAddress();
  const [faucetStatus, setFaucetStatus] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string>();
  const [claimGas, setClaimGas] = useState(0);
  const [UTXOs, setUTXOs] = useState<Array<string> | null>(null);

  const { data: inviter } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_type: inviterConf,
    },
  });

  const { data, isPending, refetch } = useRoochClientQuery(
    'getBalance',
    {
      owner: viewAddress?.genRoochAddress()!,
      coinType: '0x3::gas_coin::RGas',
    },
    { refetchInterval: 5000 }
  );

  useEffect(() => {
    if (!viewAddress) {
      return;
    }
    setFaucetStatus(true);
    client
      .queryUTXO({
        filter: {
          owner: viewAddress.toStr(),
        },
      })
      .then(async (result) => {
        const utxoIds = result.data.map((item) => item.id);
        if (utxoIds) {
          setUTXOs(utxoIds);
          const result = await client.executeViewFunction({
            target: `${faucetAddress}::gas_faucet::check_claim`,
            args: [
              Args.objectId(faucetObject),
              Args.address(viewAddress.genRoochAddress()!),
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
  }, [client, faucetAddress, faucetObject, viewAddress]);

  const fetchFaucet = async () => {
    if (errorMsg === ALREADY_CLAIMED) {
      router.push(paths.dashboard['gas-swap']);
      return;
    }

    setFaucetStatus(true);

    if (
      inviterAddress &&
      inviter &&
      inviter.data.length > 0 &&
      inviter.data[0].decoded_value?.value.is_open === true
    ) {
      try {
        const tx = new Transaction();
        tx.callFunction({
          target: `${inviterCA}::invitation::claim_from_faucet`,
          args: [
            Args.object({
              address: '0x3',
              module: 'gas_coin',
              name: 'RGas',
            }),
            Args.object({
              address: inviterCA,
              module: 'invitation',
              name: 'InvitationConf',
            }),
            Args.address(viewAddress?.genRoochAddress()!),
            Args.vec('objectId', UTXOs!),
            Args.address(inviterAddress),
          ],
        });

        const result = await client.signAndExecuteTransaction({
          transaction: tx,
          signer: session!
        })
        if (result.execution_info.status.type === 'executed') {
          refetch()
          toast.success('claim success')
          window.localStorage.setItem(INVITER_ADDRESS_KEY, '')
        }
      } catch (e) {
        console.log(e);
      } finally {
        setFaucetStatus(false);
      }
    }
  };

  return (
    <DashboardContent maxWidth="xl">
      <Card>
        <CardHeader title="Gas Faucet" sx={{ mb: 1 }} />
        <CardContent className="!pt-0">
          <Stack spacing={2}>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip className="w-fit" label="Claim Address:" variant="soft" color="default" />
              <Box className="text-gray-400 text-sm font-medium">({viewAddress?.toStr()})</Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip className="w-fit" label="Claim Rooch Address:" variant="soft" color="default" />
              <Box className="text-gray-400 text-sm font-medium">
                ({viewAddress?.genRoochAddress().toStr()})
              </Box>
            </Stack>
            <Stack direction="row" alignItems="center" spacing={0.5}>
              <Chip className="w-fit" label="RGas Balance:" variant="soft" color="secondary" />
              <Box className="text-gray-400 text-sm font-medium">
                {formatCoin(Number(data?.balance || 0), data?.decimals || 0, 2)}
              </Box>
            </Stack>
            {errorMsg
              ? errorMsg === ALREADY_CLAIMED
                ? 'You Already Claimed RGAS'
                : 'You cannot claim gas, Please make sure the current address has a valid utxo and try again'
              : ''}
            <SessionKeyGuardButton>
              <LoadingButton
                variant="soft"
                color="primary"
                disabled={errorMsg !== undefined && errorMsg !== ALREADY_CLAIMED}
                loading={isPending || faucetStatus}
                onClick={fetchFaucet}
              >
                {errorMsg === ALREADY_CLAIMED
                  ? 'Purchase RGas'
                  : errorMsg || `Claim: ${claimGas} RGas`}
              </LoadingButton>
            </SessionKeyGuardButton>
          </Stack>
        </CardContent>
      </Card>
    </DashboardContent>
  );
}
