// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useState, useEffect, useRef, useMemo } from 'react'

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
import CustomPagination from '@/components/custom-pagination.tsx'
import { CursorType } from 'src/common/interface.ts'

type StakeCardProps = {
  tokenInfo: TokenInfo | undefined,
  tokenAddress: string
}

type StakeInfo = {
  stake: boolean,
  amount: number
}

export const ActionCard: React.FC<StakeCardProps> = ({ tokenInfo, tokenAddress }) => {
  const { toast } = useToast()
  const { wallet } = useCurrentWallet()
  const client = useRoochClient()
  const { mutateAsync: signAndExecuteTransaction } = UseSignAndExecuteTransaction()

  const [loading, setLoading] = useState(false)
  const [curUTXOStakeInfo, setCurUTXOStakeInfo] = useState<StakeInfo | undefined>(undefined)
  const [selectedUTXO, setSelectUTXO] = useState('')
  const [paginationModel, setPaginationModel] = useState({ page: 0, pageSize: 10 })
  const mapPageToNextCursor = useRef<{ [page: number]: CursorType }>({})

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1] || undefined,
      pageSize: paginationModel.pageSize.toString(),
    }),
    [paginationModel],
  )
  console.log(tokenInfo)

  const { data: utxos, isPending: utxosIsPending } = useRoochClientQuery('queryUTXO', {
    filter: {
      owner: wallet!.getBitcoinAddress().toStr(),
    },
    cursor: queryOptions.cursor,
    limit: queryOptions.pageSize,
  })

  const handlePageChange = (selectedPage: number) => {
    if (selectedPage < 0) return

    setPaginationModel({
      page: selectedPage,
      pageSize: paginationModel.pageSize,
    })
  }

  const toggleUTXOSelected = (utxoId: string) => {
    if (utxoId !== selectedUTXO) {
      setSelectUTXO(utxoId)
      setCurUTXOStakeInfo(undefined)
    }
  }

  const handleStakeOrClaim = async () => {

    setLoading(true)
    const tx = new Transaction()
    tx.callFunction({
      target: `${tokenAddress}::hold_farmer::${curUTXOStakeInfo ? 'harvest' : 'stake'}`,
      args: [Args.objectId(selectedUTXO)],
    })

    try {
      const result = await signAndExecuteTransaction({
        transaction: tx,
      })

      if (result.execution_info.status.type === 'executed') {
        setSelectUTXO('')
        toast({
          title: `${curUTXOStakeInfo ? 'Claim' : 'Stake'} Successful ✅`,
          description: (
            // eslint-disable-next-line jsx-a11y/anchor-is-valid
            <a className="text-muted-foreground hover:underline cursor-pointer">
              See the transaction on explorer
            </a>
          ),
          action: <ToastAction altText="Confirm">Confirm</ToastAction>,
        })
      }
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (!utxos) {
      return
    }

    if (utxos) {
      mapPageToNextCursor.current[paginationModel.page] = utxos.next_cursor ?? null
    }
  }, [paginationModel, utxos])

  useEffect(() => {

    if (selectedUTXO === '') {
      return
    }

    setLoading(true)
    client.executeViewFunction({
      target: `${tokenAddress}::hold_farmer::check_asset_is_staked`,
      args: [Args.objectId(selectedUTXO)],
    }).then((s) => {
      if (s.vm_status === 'Executed') {
        const result = s.return_values as any[]
        const stake = result[0].decoded_value as boolean
        const amount = result[1].decoded_value as number

        if (stake) {
          setCurUTXOStakeInfo({
            stake: stake,
            amount: amount,
          })
        }
      }
    }).finally(() => setLoading(false))

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
        <Card key="info" className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-zinc-100 flex flex-row items-center justify-between">
            <div>
              <CardTitle>My Bitcoin UTXO</CardTitle>
              <CardDescription className="dark:text-blue-50/70 mt-2">
                Select your UTXO to stake or claim
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            {utxos && tokenInfo ? <>
              <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                {
                  utxos.data.map((utxo) =>
                    <UtxoCard key={utxo.tx_order} utxo={utxo} selected={utxo.id === selectedUTXO}
                              selectedCallback={toggleUTXOSelected}
                              noData={false} />,
                  )
                }
              </div>
              <CustomPagination
                currentPage={paginationModel.page}
                hasNextPage={utxos.has_next_page}
                onPageChange={handlePageChange}
              />
            </> : <UtxoCard key="loading-utxo" utxo={undefined} selected={false} selectedCallback={toggleUTXOSelected}
                            noData={!utxosIsPending && tokenInfo !== undefined} />}
          </CardContent>

        </Card>
        {
          selectedUTXO ? <>
              {
                curUTXOStakeInfo ?
                  <Card key="clam info" className="border-border/40 shadow-inner bg-border/10 dark:bg-border/60 mt-6">
                    <CardHeader className="dark:text-zinc-100">
                      <CardTitle>Claim Overview</CardTitle>
                    </CardHeader>
                    <CardContent className="text-sm dark:text-primary w-full px-6">
                        <div className="flex flex-col items-start justify-start gap-3">
                          <div
                            className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
                            <div className="w-36">
                              <span>Amount:</span>
                            </div>
                            <span
                              className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                {curUTXOStakeInfo.amount}
              </span>
                          </div>
                        </div>
                    </CardContent>
                  </Card> : <></>
              }
                <Button className="rounded-lg w-full mt-4 mb-2 md:mt-8"
                        disabled={(tokenInfo?.finished && !curUTXOStakeInfo) || loading}
                        onClick={handleStakeOrClaim}>{ loading ? 'Loading' : curUTXOStakeInfo ? 'Claim' : tokenInfo?.finished ? 'Finished' : 'Stake'}
                </Button>
            </>
            : <></>
        }
      </div>
    </div>

  )
}
