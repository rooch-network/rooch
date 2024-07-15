// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react'
import { CheckCircle2 } from 'lucide-react'
import { UTXOStateView } from '@roochnetwork/rooch-sdk'
import { cn } from '@/lib/utils'
import { CardHeader, CardContent, Card } from '@/components/ui/card'
import Skeleton from 'react-loading-skeleton'

type UtxoCardProps = {
  utxo: UTXOStateView | undefined,
  selected: boolean,
  selectedCallback: (id: string) => void,
  noData: boolean
}

export const UtxoCard: React.FC<UtxoCardProps> = ({utxo, selected, noData, selectedCallback}) => {
  return (
    <Card
      key={utxo?.id || ''}
      onClick={() => utxo && selectedCallback(utxo.id)}
      className={cn(
        'relative rounded-lg border border-border/40 dark:bg-zinc-800/90 overflow-hidden select-none',
        selected
          ? 'border-teal-400 dark:border-teal-500 bg-teal-50 dark:bg-teal-800/60'
          : '',
        // isSwitchOn && utxo.isStaked ? 'opacity-50' : 'opacity-100',
        false ? 'opacity-50 dark:bg-zinc-900' : '',
      )}
    >
      {false && (
        <div className="absolute top-0 left-0 px-5 py-0.5 bg-gradient-to-r bg-clip-padding from-teal-500 via-purple-500 to-orange-500 text-white text-xs font-semibold transform -rotate-45 -translate-x-6 translate-y-2">
          Staked
        </div>
      )}
      <CardHeader className="flex items-center justify-center">
        {utxo ? <h3 className="text-2xl">UTXO #{utxo.tx_order}</h3> : noData ? 'No UTXO' : <Skeleton width={100}/> }
      </CardHeader>
      <CardContent className="flex items-center justify-center">
        {utxo ? `Sats ${utxo.value?.value}` :  noData ? '' : <Skeleton width={100}/> }

      </CardContent>
      {selected && (
        <div className="absolute top-2 right-2">
          <CheckCircle2
            className={cn(
              'w-5 h-5 text-muted-foreground',
              true ? 'text-teal-400' : '',
            )}
          />
        </div>
      )}
    </Card>
  )
}