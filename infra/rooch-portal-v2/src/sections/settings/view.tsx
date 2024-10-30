'use client';

import axios from 'axios';
import { Args } from '@roochnetwork/rooch-sdk';
import { useState, useEffect, useCallback } from 'react';
import { CopyToClipboard } from 'react-copy-to-clipboard';
import {
  useRoochClient,
  useCurrentAddress,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { Card, Chip, Stack, TextField, CardHeader, Typography, CardContent } from '@mui/material';

import { useRouter } from 'src/routes/hooks';

import { sleep } from 'src/utils/common';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { TWITTER_ORACLE_PACKAGE_ID } from './constant';
import SessionKeysTableCard from './components/session-keys-table-card';

export function SettingsView() {
  const address = useCurrentAddress();
  const router = useRouter();

  const client = useRoochClient();

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

  const [twitterId, setTwitterId] = useState('');

  const [verifying, setVerifying] = useState(false);

  const getBindingTwitterId = useCallback(async () => {
    if (!address) {
      return;
    }
    const res = await client.executeViewFunction({
      address: TWITTER_ORACLE_PACKAGE_ID,
      module: 'twitter_account',
      function: 'resolve_author_id_by_address',
      args: [Args.address(address.toStr())],
    });
    setTwitterId((res.return_values?.[0].decoded_value as any).value.vec[0]);
    // eslint-disable-next-line consistent-return
    return (res.return_values?.[0].decoded_value as any)?.value.vec[0];
  }, [address, client]);

  useEffect(() => {
    getBindingTwitterId();
  }, [getBindingTwitterId]);

  const [tweetId, setTweetId] = useState('');

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
      <Card className="mt-4">
        <CardHeader
          title={
            <Stack direction="row" spacing={1.5} alignItems="center">
              Twitter Binding <Iconify icon="logos:twitter" />
            </Stack>
          }
          subheader="Bind a Twitter account to a Bitcoin address via publishing a tweet"
        />
        <CardContent className="!pt-2">
          {twitterId ? (
            <Chip
              className="justify-start w-fit"
              color="success"
              label={
                <Stack direction="row" spacing={1.5} alignItems="center">
                  <Iconify icon="solar:check-circle-bold" />
                  Twitter Id: {twitterId}
                </Stack>
              }
            />
          ) : (
            <Stack className="mt-2" spacing={1.5}>
              <Stack spacing={1.5}>
                <Stack className="font-medium" direction="row" spacing={0.5}>
                  1. Post the following text to you twitter account
                  <span className="font-normal text-sm text-gray-400">(Click it to copy)</span>
                </Stack>
                {address && (
                  <CopyToClipboard
                    text={`BTC:${address.toStr()} #RoochNetwork`}
                    onCopy={() => toast.success('Copy success')}
                  >
                    <Chip
                      variant="soft"
                      color="default"
                      className="w-fit font-semibold cursor-pointer"
                      label={`BTC:${address.toStr()} #RoochNetwork`}
                    />
                  </CopyToClipboard>
                )}
              </Stack>
              <Stack spacing={1.5}>
                <Stack className="font-medium">
                  2. paste the link of the tweet into the input box below
                </Stack>
                <TextField
                  size="small"
                  className="w-full"
                  value={tweetId}
                  placeholder="https://x.com/RoochNetwork/status/180000000000000000"
                  onChange={(e) => {
                    setTweetId(e.target.value);
                  }}
                />
              </Stack>
              <LoadingButton
                disabled={
                  !tweetId ||
                  (() => {
                    try {
                      const url = new URL(tweetId);
                      return url.hostname !== 'x.com';
                    } catch {
                      return true;
                    }
                  })()
                }
                color="inherit"
                loading={verifying}
                className="mt-2 w-fit"
                variant="contained"
                onClick={async () => {
                  try {
                    setVerifying(true);
                    const match = tweetId.match(/status\/(\d+)/);
                    if (match) {
                      const pureTweetId = match[1];
                      const res = await axios.post(
                        'http://test-faucet.rooch.network/fetch-tweet',
                        {
                          tweet_id: pureTweetId,
                        },
                        {
                          headers: {
                            'Content-Type': 'application/json',
                          },
                        }
                      );
                      console.log('🚀 ~ file: view.tsx:190 ~ onClick={ ~ res:', res);
                      if (res?.data?.ok) {
                        await axios.post(
                          'http://test-faucet.rooch.network/verify-and-binding-twitter-account',
                          {
                            tweet_id: pureTweetId,
                          },
                          {
                            headers: {
                              'Content-Type': 'application/json',
                            },
                          }
                        );
                      }
                      await sleep(3000);
                      const checkRes = await getBindingTwitterId();
                      if (checkRes) {
                        toast.success('Binding success');
                      }
                    }
                  } catch (error) {
                    console.log('🚀 ~ file: view.tsx:211 ~ onClick={ ~ error:', error);
                    toast.error(error.response.data.error);
                  } finally {
                    setVerifying(false);
                  }
                }}
              >
                Verify and bind Twitter account
              </LoadingButton>
            </Stack>
          )}
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
