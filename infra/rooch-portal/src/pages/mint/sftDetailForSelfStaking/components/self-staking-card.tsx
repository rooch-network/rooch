// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useState, useEffect } from 'react'

import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

import { cn } from '@/lib/utils'
import { CheckCircle2 } from 'lucide-react'
import { useToast } from '@/components/ui/use-toast'
import { ToastAction } from '@/components/ui/toast'
import { UTXO } from '@/common/interface'
import { useRoochClientQuery, useCurrentWallet, useRoochClient } from '@roochnetwork/rooch-sdk-kit'
import Skeleton, { SkeletonTheme } from 'react-loading-skeleton'
import { TokenInfo } from '@/pages/mint/util/get-token-info'
import { UtxoCard } from '@/pages/mint/sftDetailForSelfStaking/components/utxo'
import { Transaction, Args } from '@roochnetwork/rooch-sdk'
import { UseSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit'

type StakeCardProps = {
  tokenInfo: TokenInfo | undefined,
  tokenAddress: string
  inProgress: boolean
}

export const SelfStakingCard:React.FC<StakeCardProps> = ({tokenInfo, tokenAddress, inProgress}) => {
  const { toast } = useToast()
  const {wallet} = useCurrentWallet()

  console.log(tokenInfo)

  const client = useRoochClient()
  const {mutateAsync: signAndExecuteTransaction} = UseSignAndExecuteTransaction()
  console.log(tokenInfo)
  const [selectedUTXO, setSelectUTXO] = useState('')

  const toggleUTXOSelected = (utxoId: string) => {
    // if (utxoId === )
    setSelectUTXO(utxoId)
    // setUtxos(
    //   utxos.map((utxo) => {
    //     if (utxo.id === utxoId && !utxo.isStaked) {
    //       return { ...utxo, isSelected: !utxo.isSelected }
    //     }
    //     return utxo
    //   }),
    // )
  }

  const handleSelfStake = async () => {

    const tx = new Transaction()
    tx.callFunction({
      target: `${tokenAddress}::hold_farmer::stake`,
      args: [Args.objectId(selectedUTXO)]
    })

    const result = await signAndExecuteTransaction({
      transaction: tx
    })

    console.log(result)

    // const hasSelectedUTXOs = utxos.some((utxo) => utxo.isSelected) // For displaying "success" message

    // setUtxos(
    //   utxos.map((utxo) => {
    //     if (utxo.isSelected) {
    //       return { ...utxo, isStaked: true, isSelected: false }
    //     }
    //     return utxo
    //   }),
    // )
    //
    // if (hasSelectedUTXOs) {
    //   toast({
    //     title: 'Self-staking successful ✅',
    //     description: (
    //       // eslint-disable-next-line jsx-a11y/anchor-is-valid
    //       <a className="text-muted-foreground hover:underline cursor-pointer">
    //         See the transaction on explorer
    //       </a>
    //     ),
    //     action: <ToastAction altText="Confirm">Confirm</ToastAction>,
    //   })
    // }
  }

  const {data:utxos, isPending: utxosIsPending} = useRoochClientQuery('queryUTXO', {
    filter: {
      owner: wallet?.getBitcoinAddress().toStr() || ''
    }
  })
  useEffect(() => {
    // const {data: state} = useRoochClientQuery('getStates', {
    //   accessPath: `/resource/${wallet?.getRoochAddress().toHexAddress()}/${tokenAddress}::hold_farmer::UserStake`,
    //   stateOption: {
    //     decode: true,
    //     showDisplay: true
    //   }
    // })
  }, [])

  console.log(utxos)

  return (
    <div className="mt-6">
      <div className="h-full w-full">
        <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-zinc-100 flex flex-row items-center justify-between">
            <div>
              <CardTitle>My Bitcoin UTXO</CardTitle>
              <CardDescription className="dark:text-blue-50/70">
                Stake your UTXO below
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {utxos && tokenInfo ? utxos.data.map((utxo) =>
                <UtxoCard utxo={utxo} selected={utxo.object_id === selectedUTXO} selectedCallback={toggleUTXOSelected} noData={false}/>
              ): <UtxoCard utxo={undefined} selected={false} selectedCallback={toggleUTXOSelected} noData={!utxosIsPending && tokenInfo !== undefined}/>}
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/40 shadow-inner bg-border/10 dark:bg-border/60 mt-6">
          <CardHeader className="dark:text-zinc-100">
            <CardTitle>Mint Overview</CardTitle>
          </CardHeader>
          <CardContent className="text-sm dark:text-primary w-full px-6">
            <SkeletonTheme baseColor="#27272A" highlightColor="#444">
              <div className="flex flex-col items-start justify-start gap-3">
            <div
              className="flex items-center justify-start gap-6 text-sm text-muted-foreground/75 dark:text-muted-foreground">
              <div className="w-36">
                <span>Type:</span>
              </div>
              <span
                className="border border-accent dark:border-none dark:bg-zinc-800 py-0.5 px-2 rounded-lg text-gray-800 dark:text-gray-50 tracking-tight ">
                aaa
              </span>
            </div>
              </div></SkeletonTheme>
          </CardContent>
        </Card>

      </div>
      <Button className="rounded-lg w-full mt-4 mb-2 md:mt-8" onClick={handleSelfStake}>{inProgress ? 'Mint' : 'Claim'}</Button>
    </div>

  )
}
