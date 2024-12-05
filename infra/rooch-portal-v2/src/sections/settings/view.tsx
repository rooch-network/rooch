'use client'

import axios from 'axios'
import { useState, useEffect, useCallback } from 'react'
import { CopyToClipboard } from 'react-copy-to-clipboard'
import { Args, Transaction, stringToBytes } from '@roochnetwork/rooch-sdk'
import {
  useRoochClient,
  useCurrentAddress,
  useCurrentNetwork,
  useCurrentSession,
  useRoochClientQuery
} from '@roochnetwork/rooch-sdk-kit'

import { LoadingButton } from '@mui/lab'
import { Card, Chip, Stack, TextField, CardHeader, Typography, CardContent } from '@mui/material'

import { sleep } from 'src/utils/common'

import { DashboardContent } from 'src/layouts/dashboard'

import { toast } from 'src/components/snackbar'
import { Iconify } from 'src/components/iconify'

import { useNetworkVariable } from '../../hooks/use-networks'
import SessionKeysTableCard from './components/session-keys-table-card'
import SessionKeyGuardButtonV1 from '../../components/auth/session-key-guard-button-v1'

export function SettingsView() {
  const address = useCurrentAddress()

  const session = useCurrentSession()
  const client = useRoochClient()
  const network = useCurrentNetwork()
  const faucetUrl = useNetworkVariable('faucetUrl')
  const twitterOracleAddress = useNetworkVariable('twitterOracleAddress')
  const [tweetStatus, setTweetStatus] = useState('')
  const [twitterId, setTwitterId] = useState<string>()
  const [verifying, setVerifying] = useState(false)

  const {
    data: sessionKeys,
    isPending: isLoadingSessionKeys,
    refetch: refetchSessionKeys,
  } = useRoochClientQuery('getSessionKeys', {
    address: address!.genRoochAddress().toHexAddress()
    }
  )

  const fetchTwitterId = useCallback(async () => {
    if (!address) {
      return
    }
    const res = await client.executeViewFunction({
      address: twitterOracleAddress,
      module: 'twitter_account',
      function: 'resolve_author_id_by_address',
      args: [Args.address(address.toStr())],
    })
    let _twitterId: string | undefined
    if (res.vm_status === 'Executed') {
      if (res.return_values?.[0].value.value !== '0x00') {
        _twitterId = (res.return_values?.[0].decoded_value as any).value.vec
          .value[0][0] as string;
        _twitterId = new TextDecoder('utf-8').decode(
          stringToBytes('hex', _twitterId.replace('0x', ''))
        );

        setTwitterId(_twitterId);
      }
    }
    // eslint-disable-next-line consistent-return
    return _twitterId
  }, [address, client, twitterOracleAddress])

  useEffect(() => {
    fetchTwitterId()
  }, [fetchTwitterId])

  const disconnectTwitter = async () => {
    if (!session) {
      return
    }
    try {
      const tx = new Transaction()
      tx.callFunction({
        target: `${twitterOracleAddress}::twitter_account::unbinding_twitter_account`
      })

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: session
      })

      if (result.execution_info.status.type === 'executed') {
        setTwitterId(undefined)
        await fetchTwitterId()
        toast.success('Disconnect twitter success')
      } else {
        toast.error('Disconnect twitter aborted')
      }
    } catch (e) {
      toast.error(e.message)
    }
  }

  const networkText = network === 'mainnet' ? 'Pre-mainnet' : 'Testnet'
  const XText = `BTC:${address?.toStr()} 

Rooch ${networkText} is live! Bind your Twitter to earn  RGas, and visit https://${network === 'mainnet' ? '':'test-'}grow.rooch.network to earn rewards with your BTC. 

Join Rooch:
https://${network === 'mainnet' ? '':'test-'}portal.rooch.network/inviter/${address?.genRoochAddress().toBech32Address()}

#RoochNetwork #${networkText}`

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
              Twitter Binding <Iconify icon="logos:twitter" />
            </Stack>
          }
          subheader="Bind a Twitter account to a Bitcoin address via publishing a tweet"
        />
        <CardContent className="!pt-2">
          {twitterId ? (
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

            <SessionKeyGuardButtonV1 desc="Disconnect Twitter" callback={disconnectTwitter}/>
            </Stack>
          ) : (
            <Stack className="mt-2" spacing={1.5}>
              <Stack spacing={1.5}>
                <Stack className="font-medium" direction="row" spacing={0.5}>
                  1. Post the following text to you twitter account
                  <span className="font-normal text-sm text-gray-400">(Click it to twitter)</span>
                </Stack>
                {address && (
                  <CopyToClipboard
                    text={XText}
                    onCopy={() => {
                      window.open(
                        `https://twitter.com/intent/tweet?text=${encodeURIComponent(XText)}`,
                        '_blank',
                      )
                    }}
                  >
                    <Stack
                      className="font-medium cursor-pointer text-wrap bg-gray-200 p-3 rounded-md whitespace-pre-line">
                      {XText}
                    </Stack>
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
                  value={tweetStatus}
                  placeholder="https://x.com/RoochNetwork/status/180000000000000000"
                  onChange={(e) => {
                    setTweetStatus(e.target.value)
                  }}
                />
              </Stack>
              <Stack spacing={1.5}>
                <Stack className="font-medium">
                  🔔Tips: If you just posted a twitter message, Wait for on-chain synchronization (2-3 minutes)
                </Stack>
              </Stack>
              <LoadingButton
                disabled={
                  !tweetStatus ||
                  (() => {
                    try {
                      const url = new URL(tweetStatus)
                      return url.hostname !== 'x.com'
                    } catch {
                      return true
                    }
                  })()
                }
                color="inherit"
                loading={verifying}
                className="mt-2 w-fit"
                variant="contained"
                onClick={async () => {
                  try {
                    setVerifying(true)
                    const match = tweetStatus.match(/status\/(\d+)/)
                    if (match) {
                      const pureTweetId = match[1]
                      const res = await axios.post(
                        `${faucetUrl}/fetch-tweet`,
                        {
                          tweet_id: pureTweetId,
                        },
                        {
                          headers: {
                            'Content-Type': 'application/json',
                          },
                        },
                      )
                      console.log('🚀 ~ file: view.tsx:190 ~ onClick={ ~ res:', res)
                      if (res?.data?.ok) {
                        await axios.post(
                          `${faucetUrl}/verify-and-binding-twitter-account`,
                          {
                            tweet_id: pureTweetId,
                          },
                          {
                            headers: {
                              'Content-Type': 'application/json',
                            },
                          },
                        )
                      }
                      await sleep(3000)
                      const checkRes = await fetchTwitterId()
                      if (checkRes) {
                        toast.success('Binding success')
                      }
                    }
                  } catch (error) {
                    console.log('🚀 ~ file: view.tsx:211 ~ onClick={ ~ error:', error)
                    toast.error(error.response.data.error)
                  } finally {
                    setVerifying(false)
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
  )
}
