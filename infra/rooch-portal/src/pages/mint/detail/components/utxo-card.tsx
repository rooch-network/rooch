// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react'
import { CheckCircle2 } from 'lucide-react'
import Skeleton from 'react-loading-skeleton'

import { UTXOStateView } from '@roochnetwork/rooch-sdk'

import { cn } from '@/lib/utils'
import { CardHeader, CardContent, Card } from '@/components/ui/card'

type UtxoCardProps = {
  utxo: UTXOStateView | undefined
  selected: boolean
  selectedCallback: (id: string) => void
}

export const UtxoCard: React.FC<UtxoCardProps> = ({ utxo, selected, selectedCallback }) => {
  return (
    <Card
      onClick={() => utxo && selectedCallback(utxo.id)}
      className={cn(
        'relative rounded-lg border border-border/40 dark:bg-zinc-800/90 overflow-hidden select-none',
        selected ? 'border-teal-400 dark:border-teal-500 bg-teal-50 dark:bg-teal-800/60' : '',
      )}
    >
      <CardHeader className="flex items-center justify-center">
        {utxo ? <h3 className="text-2xl">UTXO #{utxo.tx_order}</h3> : <Skeleton width={100} />}
      </CardHeader>
      <CardContent className="flex items-center justify-center">
        {utxo ? `Sats ${utxo.value?.value}` : <Skeleton width={100} />}
      </CardContent>
      {selected && (
        <div className="absolute top-2 right-2">
          <CheckCircle2 className={cn('w-5 h-5 text-muted-foreground', 'text-teal-400')} />
        </div>
      )}
    </Card>
  )
}
