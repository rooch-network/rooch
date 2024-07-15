// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useState, useEffect } from 'react'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'

import { Transaction, Args } from '@roochnetwork/rooch-sdk'
import {
  useRoochClientQuery,
  useCurrentWallet,
  UseSignAndExecuteTransaction,
  useRoochClient,
} from '@roochnetwork/rooch-sdk-kit'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useToast } from '@/components/ui/use-toast'
import { ToastAction } from '@/components/ui/toast'
import { TokenInfo } from '@/pages/mint/util/get-token-info'
import { UtxoCard } from './utxo-card.tsx'

type StakeCardProps = {
  tokenInfo: TokenInfo | undefined,
  tokenAddress: string
}

export const ActionCard: React.FC<StakeCardProps> = ({ tokenInfo, tokenAddress }) => {
  const { toast } = useToast()
  const { wallet } = useCurrentWallet()

  const client = useRoochClient()

  const { mutateAsync: signAndExecuteTransaction } = UseSignAndExecuteTransaction()
  const [award, setAward] = useState(0)
  const [fetchAwardStatus, setFetchAwardStatus] = useState(false)
  const [selectedUTXO, setSelectUTXO] = useState('')

  const toggleUTXOSelected = (utxoId: string) => {
    if (utxoId !== selectedUTXO) {
      setSelectUTXO(utxoId)
      setAward(0)
    }
  }

  const handleStakeOrClaim = async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${tokenAddress}::hold_farmer::${award > 0 ? 'harvest' : 'stake'}`,
      args: [Args.objectId(selectedUTXO)],
    })

    const result = await signAndExecuteTransaction({
      transaction: tx,
    })

    if (result.execution_info.status.type === 'executed') {
      toast({
        title: `${award > 0 ? 'Claim' : 'Stake'} Successful ✅`,
        description: (
          // eslint-disable-next-line jsx-a11y/anchor-is-valid
          <a className="text-muted-foreground hover:underline cursor-pointer">
            See the transaction on explorer
          </a>
        ),
        action: <ToastAction altText="Confirm">Confirm</ToastAction>,
      })
    }
  }

  const { data: utxos, isPending: utxosIsPending } = useRoochClientQuery('queryUTXO', {
    filter: {
      owner: wallet?.getBitcoinAddress().toStr() || '',
    },
  })

  useEffect(() => {

    setFetchAwardStatus(true)
    client.executeViewFunction({
      target: `${tokenAddress}::hold_farmer::query_gov_token_amount`,
      args: [Args.objectId(selectedUTXO)],
    }).then((s) => {
      if (s.vm_status === 'Executed') {
        setAward(s.return_values![0].decoded_value as number)
      }
    }).finally(() => setFetchAwardStatus(false))

  }, [client, selectedUTXO, tokenAddress])

  // useEffect(() => {
  //
  //   client.getStates({
  //     accessPath: `/resource/${wallet?.getRoochAddress().toHexAddress()}/${tokenAddress}::hold_farmer::UserStake`,
  //     stateOption: {
  //       decode: true,
  //       showDisplay: true
  //     }
  //   }).then(async (result) => {
  //     // Maybe we should define the type?
  //     const tableId = (
  //       (
  //         ((result[0].decoded_value as AnnotatedMoveStructView).value['stake'] as AnnotatedMoveStructView)
  //       ).value['handle'] as AnnotatedMoveStructView
  //     ).value['id'] as string
  //
  //     const tablePath = `/table/${tableId}`
  //
  //     const statePage = await client.listStates({
  //       accessPath: tablePath,
  //       cursor: '',
  //       limit: '1000', // TODO: support page?
  //       stateOption: {
  //         decode: true,
  //       },
  //     })
  //
  //     console.log(statePage)
  //   })
  //
  // }, [wallet, tokenAddress, client])

  return (
    <div className="mt-6">
      <div className="h-full w-full">
        <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-zinc-100 flex flex-row items-center justify-between">
            <div>
              <CardTitle>My Bitcoin UTXO</CardTitle>
              <CardDescription className="dark:text-blue-50/70 mt-2">
                Select your UTXO to stake or claim
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {utxos && tokenInfo ? utxos.data.map((utxo) =>
                <UtxoCard utxo={utxo} selected={utxo.id === selectedUTXO} selectedCallback={toggleUTXOSelected}
                          noData={false} />,
              ) : <UtxoCard utxo={undefined} selected={false} selectedCallback={toggleUTXOSelected}
                            noData={!utxosIsPending && tokenInfo !== undefined} />}
            </div>
          </CardContent>
        </Card>
        {
          selectedUTXO ? <>
              {
                award > 0 ? <Card className="border-border/40 shadow-inner bg-border/10 dark:bg-border/60 mt-6">
                  <CardHeader className="dark:text-zinc-100">
                    <CardTitle>Claim Overview</CardTitle>
                  </CardHeader>
                  <CardContent className="text-sm dark:text-primary w-full px-6">
                    <SkeletonTheme baseColor="#27272A" highlightColor="#444">
                      <div className="flex flex-col items-start justify-start gap-3">
                        <div
                          className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                          <div className="w-36">
                            <span>Amount:</span>
                          </div>
                          {!fetchAwardStatus ? <span
                            className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                {award}
              </span> : <Skeleton width={150} />}
                        </div>
                      </div>
                    </SkeletonTheme>
                  </CardContent>
                </Card> : <></>
              }
              {fetchAwardStatus ? <></> :
                <Button className="rounded-lg w-full mt-4 mb-2 md:mt-8"
                        onClick={handleStakeOrClaim}>{tokenInfo!.endTime > Date.now() / 1000 && award <= 0 ? 'Mint' : 'Claim'}
                </Button>
              }
            </>
            : <></>
        }
      </div>
    </div>

  )
}
