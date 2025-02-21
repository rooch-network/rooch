'use client';

import axios from 'axios';
import { useState } from 'react';
import { CopyToClipboard } from 'react-copy-to-clipboard';
import {
  toHEX,
  Transaction,
  stringToBytes,
  BitcoinAddress,
  isValidRoochAddress,
} from '@roochnetwork/rooch-sdk';
import {
  useRoochClient,
  SessionKeyGuard,
  useCurrentWallet,
  useCurrentAddress,
  useCurrentSession,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Card,
  Chip,
  Stack,
  Button,
  TextField,
  CardHeader,
  Typography,
  CardContent,
} from '@mui/material';

import { sleep } from 'src/utils/common';

import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { INVITER_ADDRESS_KEY } from 'src/utils/inviter';
import { useNetworkVariable } from 'src/hooks/use-networks';
import { ShareTwitter } from 'src/components/twitter/share';
import useAccountTwitterId from 'src/hooks/account/use-account-twitter-id';
import SessionKeysTableCard from './components/session-keys-table-card';

export function SettingsView() {
  const address = useCurrentAddress();

  const session = useCurrentSession();
  const client = useRoochClient();
  const wallet = useCurrentWallet();
  const faucetUrl = useNetworkVariable('faucet').url;
  const twitterOracleAddress = useNetworkVariable('roochMultiSigAddr');
  const inviter = useNetworkVariable('inviter');
  const [tweetStatus, setTweetStatus] = useState('');
  const [verifying, setVerifying] = useState(false);

  const {
    data: sessionKeys,
    isPending: isLoadingSessionKeys,
    refetch: refetchSessionKeys,
  } = useRoochClientQuery('getSessionKeys', {
    address: address?.genRoochAddress().toHexAddress() || '',
  });

  const {
    data: twitterId,
    isPending: isPendingTwitterId,
    refetch: refetchTwitterId,
  } = useAccountTwitterId(address);

  const loopFetchTwitterId = async (count = 0) => {
    const id = await refetchTwitterId();

    if (id || count === 3) {
      return id;
    }

    return loopFetchTwitterId(count + 1);
  };

  const checkTwitterObj = async (id: string) => {
    const result = await client.queryObjectStates({
      filter: {
        object_id: id,
      },
    });

    if (result.data.length === 0) {
      await sleep(10000);
      return checkTwitterObj(id);
    }

    // TODO: twitter post btc address !== current wallet address.
    // if (result.data[0].owner_bitcoin_address !== address?.toStr()) {
    //   throw (new Error('The twitter post btc address does not match the wallet address'))
    // }

    return '';
  };
  const fetchTwitterPost = async (pureTweetId: string) => {
    const res = await axios.post(
      `${faucetUrl}/fetch-tweet`,
      {
        tweet_id: pureTweetId,
      },
      {
        headers: {
          'Content-Type': 'application/json',
        },
      }
    );

    if (res.data.ok) {
      await checkTwitterObj(res.data.ok);
    }
  };

  const disconnectTwitter = async () => {
    if (!session) {
      return;
    }
    try {
      const tx = new Transaction();
      tx.callFunction({
        target: `${twitterOracleAddress}::twitter_account::unbinding_twitter_account`,
      });

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: session,
      });

      if (result.execution_info.status.type === 'executed') {
        await refetchTwitterId();
        toast.success('Disconnect twitter success');
      } else {
        toast.error('Disconnect twitter aborted');
      }
    } catch (e) {
      toast.error(e.message);
    }
  };

  const bindTwitter = async (pureTweetId: string) => {
    await axios.post(
      `${faucetUrl}/verify-and-binding-twitter-account`,
      {
        tweet_id: pureTweetId,
      },
      {
        headers: {
          'Content-Type': 'application/json',
        },
      }
    );
  };

  const bindWithInviter = async (inviterAddr: string, pureTweetId: string) => {
    const signMsg = 'Welcome to use Rooch! Connect with Twitter and claim your Rgas.';
    const sign = await wallet.wallet?.sign(stringToBytes('utf8', signMsg));
    const pk = wallet.wallet!.getPublicKey().toBytes();

    const inviterRoochAddr = isValidRoochAddress(inviterAddr)
      ? inviterAddr
      : new BitcoinAddress(inviterAddr).genRoochAddress().toStr();

    const payload = JSON.stringify({
      inviter: inviterRoochAddr,
      tweet_id: pureTweetId,
      claimer_sign: toHEX(sign!),
      public_key: toHEX(pk),
      message: signMsg,
    });
    await axios.post(`${faucetUrl}/binding-twitter-with-inviter`, payload, {
      headers: {
        'Content-Type': 'application/json',
      },
    });

    window.localStorage.setItem(INVITER_ADDRESS_KEY, '');
  };

  const handleBindTwitter = async () => {
    // step 1, check twitter
    const match = tweetStatus.match(/status\/(\d+)/);

    if (!match) {
      toast.error('twitter invalid');
      return;
    }
    setVerifying(true);

    try {
      const pureTweetId = match[1];
      await fetchTwitterPost(pureTweetId);

      // step 2, check inviter
      const inviterAddr = window.localStorage.getItem(INVITER_ADDRESS_KEY);
      if (inviterAddr && inviterAddr !== '' && inviterAddr !== address?.toStr()) {
        // check invite is open
        const result = await client.queryObjectStates({
          filter: {
            object_type: inviter.obj(inviter),
          },
          queryOption: {
            decode: true,
          },
        });

        if (
          result &&
          result.data.length > 0 &&
          result.data[0].decoded_value?.value.is_open === true
        ) {
          await bindWithInviter(inviterAddr, pureTweetId);
        } else {
          await bindTwitter(pureTweetId);
        }
      } else {
        await bindTwitter(pureTweetId);
      }

      await sleep(3000);
      const checkRes = await loopFetchTwitterId();
      if (checkRes) {
        toast.success('Binding success');
      }
    } catch (error) {
      if ('response' in error) {
        if ('error' in error.response.data) {
          toast.error(error.response.data.error);
        } else {
          toast.error(error.response.data);
        }
      } else {
        toast.error(error.message);
      }
    } finally {
      setVerifying(false);
    }
  };

  return (
    <DashboardContent maxWidth="xl">
      <Card className="mt-4">
        <CardHeader
          title="Rooch Address"
          subheader="Use Rooch address in the application and smart contract development"
        />
        <CardContent className="!pt-2">
          <Stack>
            <CopyToClipboard text={address?.genRoochAddress().toStr() || ''}>
              <Chip className="justify-start w-fit" label={address?.genRoochAddress().toStr()} />
            </CopyToClipboard>
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
              Twitter Binding & Invite partners <Iconify icon="logos:twitter" />
            </Stack>
          }
          subheader="Bind a Twitter account to a Bitcoin address via publishing a tweet"
        />
        <CardContent className="!pt-2">
          {isPendingTwitterId ? (
            <></>
          ) : twitterId ? (
            <Stack className="mt-2" spacing={1.5} alignItems="flex-start">
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

              <SessionKeyGuard onClick={disconnectTwitter}>
                <Button variant="outlined">Disconnect Twitter</Button>
              </SessionKeyGuard>
            </Stack>
          ) : (
            <Stack className="mt-2" spacing={1.5}>
              <Stack spacing={1.5}>
                <Stack className="font-medium" direction="row" spacing={0.5}>
                  1. Post the following text to you twitter account
                  <span className="font-normal text-sm text-gray-400">(Click it to twitter)</span>
                </Stack>
                {address && <ShareTwitter />}
              </Stack>
              <Stack spacing={1.5}>
                <Stack className="font-medium">
                  2. paste the link of the tweet into the input box below
                </Stack>
                <TextField
                  size="small"
                  className="w-full"
                  value={tweetStatus}
                  placeholder="https://x.com/RoochNetwork/status/180000000000000000"
                  onChange={(e) => {
                    setTweetStatus(e.target.value);
                  }}
                />
              </Stack>
              <Stack spacing={1.5}>
                <Stack className="font-medium">
                  🔔Tips: If you just posted a twitter message, Wait for on-chain synchronization
                  (2-3 minutes)
                </Stack>
              </Stack>
              <LoadingButton
                disabled={
                  !tweetStatus ||
                  (() => {
                    try {
                      const url = new URL(tweetStatus);
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
                onClick={handleBindTwitter}
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
