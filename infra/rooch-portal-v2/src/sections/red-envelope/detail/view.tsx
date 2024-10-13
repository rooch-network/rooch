'use client';

import dayjs from 'dayjs';
import { useMemo } from 'react';
import { Args, Transaction, normalizeTypeArgsToStr } from '@roochnetwork/rooch-sdk';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Box,
  Card,
  Chip,
  Stack,
  Divider,
  CardHeader,
  Typography,
  CardContent,
} from '@mui/material';

import { shortAddress } from 'src/utils/address';
import { fShortenNumber } from 'src/utils/format-number';

import { RED_ENVELOPE } from 'src/config/constant';
import { DashboardContent } from 'src/layouts/dashboard';

import type { RedEnvelopeItem } from '../types';

export default function RedEnvelopeDetail({ redEnvelopeId }: { redEnvelopeId: string }) {
  const currentAddress = useCurrentAddress();
  const roochClient = useRoochClient();

  const { data: redEnvelopeObject } = useRoochClientQuery(
    'queryObjectStates',
    {
      filter: {
        object_id: redEnvelopeId,
      },
      queryOption: {
        decode: true,
      },
    },
    { refetchInterval: 5000 }
  );

  const redEnvelopeInfo = useMemo(() => {
    const info = redEnvelopeObject?.data?.[0]?.decoded_value?.value as unknown as
      | RedEnvelopeItem
      | undefined;
    return info;
  }, [redEnvelopeObject]);

  const { data: redEnvelopeSenderState } = useRoochClientQuery(
    'queryObjectStates',
    {
      filter: {
        object_id: redEnvelopeInfo?.sender || '',
      },
    },
    { enabled: !!redEnvelopeInfo?.sender }
  );

  const redEnvelopeSenderBitcoinAddress = useMemo(() => {
    if (redEnvelopeSenderState) {
      return redEnvelopeSenderState.data[0].owner_bitcoin_address;
    }
    return undefined;
  }, [redEnvelopeSenderState]);

  const { data: redEnvelopeCoinInfo } = useRoochClientQuery(
    'getBalance',
    {
      owner: currentAddress?.genRoochAddress().toStr() || '',
      coinType:
        '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC',
    },
    { enabled: !!redEnvelopeInfo?.coin_store.type }
  );

  if (!redEnvelopeInfo) {
    return null;
  }

  return (
    <DashboardContent maxWidth="xl">
      <Stack alignItems="center">
        <Card className="mt-4 w-[640px] !bg-[#f43f5e] !text-white" elevation={4}>
          <CardHeader
            className="!mb-0"
            titleTypographyProps={{ className: '!text-2xl' }}
            subheaderTypographyProps={{ className: '!text-gray-300 !text-xs' }}
            title={
              <Stack
                direction="row"
                className="w-full"
                justifyContent="space-between"
                alignItems="center"
              >
                <Box className="text-2xl">Claim this Red Envelope!</Box>
                <Chip
                  variant="soft"
                  size="small"
                  sx={{ color: '#FED047', opacity: 0.75 }}
                  className="!text-xs"
                  label={shortAddress(redEnvelopeId, 8, 6)}
                />
              </Stack>
            }
            subheader={
              <Stack direction="row" className="w-full mt-1" justifyContent="space-between">
                <Box className="text-lg">
                  Remain:
                  <Chip
                    variant="soft"
                    sx={{ color: '#D1D5DB' }}
                    className="!text-lg ml-1"
                    label={19}
                  />
                </Box>
              </Stack>
            }
            sx={{ mb: 3 }}
          />
          <CardContent>
            <Stack justifyContent="center" alignItems="center" spacing={2}>
              <Box className="text-6xl font-semibold">
                {redEnvelopeInfo.total_coin} {redEnvelopeCoinInfo?.symbol}
              </Box>
              <LoadingButton
                variant="outlined"
                color="success"
                onClick={async () => {
                  const txn = new Transaction();
                  txn.callFunction({
                    address: RED_ENVELOPE,
                    module: 'red_envelope_v3',
                    function: 'claim_coin_envelope',
                    args: [
                      // con red envelope object
                      Args.objectId(redEnvelopeId),
                    ],
                    typeArgs: [
                      normalizeTypeArgsToStr({
                        target:
                          '0xe94e9b71c161b87b32bd679aebfdd0e106cd173fefc67edf178024081f33a812::rooch_clicker_coin::RCC',
                      }),
                    ],
                  });
                  const reward = await roochClient.getEvents({
                    eventHandleType:
                      '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977::red_envelope_v3::ClaimCoinEvent',
                    eventOptions: {
                      decode: true,
                    },
                  });
                }}
              >
                Claim
              </LoadingButton>
            </Stack>
          </CardContent>
          <Divider sx={{ borderStyle: 'dashed' }} />
          <Box
            display="grid"
            gridTemplateColumns="repeat(3, 1fr)"
            sx={{ py: 3, typography: 'subtitle1' }}
          >
            <Stack alignItems="center">
              <Typography
                className="!text-gray-300"
                variant="caption"
                component="div"
                sx={{ mb: 0.5 }}
              >
                Count
              </Typography>
              <Stack className="h-[32px] items-center justify-center">
                {fShortenNumber(redEnvelopeInfo.total_envelope)}
              </Stack>
            </Stack>

            <Stack alignItems="center">
              <Typography
                className="!text-gray-300"
                variant="caption"
                component="div"
                sx={{ mb: 0.5 }}
              >
                Sender
              </Typography>

              <Stack className="h-[32px] items-center justify-center text-xs">
                {shortAddress(redEnvelopeSenderBitcoinAddress, 8, 6)}
              </Stack>
            </Stack>

            <Stack alignItems="center">
              <Typography
                className="!text-gray-300"
                variant="caption"
                component="div"
                sx={{ mb: 0.5 }}
              >
                Time
              </Typography>
              <Stack className="text-xs h-[32px] items-center justify-center">
                {dayjs(Number(redEnvelopeInfo.start_time)).format('MM/DD HH:mm')} -{' '}
                {dayjs(Number(redEnvelopeInfo.end_time)).format('MM/DD HH:mm')}
              </Stack>
            </Stack>
          </Box>
        </Card>
      </Stack>
    </DashboardContent>
  );
}
