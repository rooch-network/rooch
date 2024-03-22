import { useState } from 'react'

import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { UTXO } from '../common/utxo-interface'
import { cn } from '@/lib/utils'

const SAMPLE_UTXOS: UTXO[] = [
  { id: 0, amount: 1000, isStaked: false, isSelected: false },
  { id: 1, amount: 2000, isStaked: false, isSelected: false },
  { id: 2, amount: 1500, isStaked: true, isSelected: false },
]

export const SelfStakingCard = () => {
  const [isSwitchOn, setIsSwitchOn] = useState(false)
  const [utxos, setUtxos] = useState<UTXO[]>(SAMPLE_UTXOS)

  const handleSwitchChange = (checked: boolean) => {
    setIsSwitchOn(checked)

    if (!checked) {
      setUtxos(
        utxos.map((utxo) => ({
          ...utxo,
          isSelected: false,
        })),
      )
    }
  }

  const toggleUTXOSelected = (utxoId: number) => {
    setUtxos(
      utxos.map((utxo) => {
        if (utxo.id === utxoId && !utxo.isStaked) {
          return { ...utxo, isSelected: !utxo.isSelected }
        }
        return utxo
      }),
    )
  }

  return (
    <div className="mt-6">
      <div className="h-full w-full">
        <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-teal-100 flex flex-row items-center justify-between">
            <div>
              <CardTitle>My Bitcoin UTXO</CardTitle>
              <CardDescription className="dark:text-teal-50/70">
                Stake your UTXO below
              </CardDescription>
            </div>

            <div className="flex items-center justify-center gap-4">
              <div className="flex items-center space-x-2">
                <Switch
                  id="batch-mode"
                  checked={isSwitchOn}
                  onCheckedChange={handleSwitchChange}
                  className="data-[state=checked]:bg-teal-600 dark:data-[state=checked]:bg-teal-400"
                />
                <Label htmlFor="batch-mode" className="text-muted-foreground">
                  Batch Mode
                </Label>
              </div>
              <Button size="default" className="rounded-lg">
                Self-stake
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {utxos.map((utxo) => (
                <Card
                  key={utxo.id}
                  onClick={() => isSwitchOn && toggleUTXOSelected(utxo.id)}
                  className={cn(
                    'rounded-lg border border-border/40 dark:bg-zinc-800/90 overflow-hidden select-none',
                    utxo.isSelected
                      ? 'border-teal-400 dark:border-teal-500 bg-teal-50 dark:bg-teal-800/60'
                      : '',
                  )}
                >
                  <CardHeader className="flex items-center justify-center">
                    <h3 className="text-2xl">UTXO #{utxo.id}</h3>
                  </CardHeader>
                  <CardContent className="flex items-center justify-center">
                    Amount {utxo.amount}
                  </CardContent>
                </Card>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
