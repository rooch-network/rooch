import { useState } from 'react'

import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { UTXO } from '../common/utxo-interface'
import { cn } from '@/lib/utils'
import { CheckCircle2 } from 'lucide-react'
import { useToast } from '@/components/ui/use-toast'
import { ToastAction } from '@/components/ui/toast'

const SAMPLE_UTXOS: UTXO[] = [
  { id: 0, amount: 1000, isStaked: false, isSelected: false },
  { id: 1, amount: 2000, isStaked: false, isSelected: false },
  { id: 2, amount: 2000, isStaked: false, isSelected: false },
  { id: 3, amount: 1500, isStaked: true, isSelected: false },
]

export const SelfStakingCard = () => {
  const { toast } = useToast()
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

  const handleSelfStake = () => {
    const hasSelectedUTXOs = utxos.some((utxo) => utxo.isSelected) // For displaying "success" message

    setUtxos(
      utxos.map((utxo) => {
        if (utxo.isSelected) {
          return { ...utxo, isStaked: true, isSelected: false }
        }
        return utxo
      }),
    )

    if (hasSelectedUTXOs) {
      toast({
        title: 'Self-staking successful ✅',
        description: (
          <a className="text-muted-foreground hover:underline cursor-pointer">
            See the transaction on explorer
          </a>
        ),
        action: <ToastAction altText="Confirm">Confirm</ToastAction>,
      })
    }
  }

  return (
    <div className="mt-6">
      <div className="h-full w-full">
        <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-blue-100 flex flex-row items-center justify-between">
            <div>
              <CardTitle>My Bitcoin UTXO</CardTitle>
              <CardDescription className="dark:text-blue-50/70">
                Stake your UTXO below
              </CardDescription>
            </div>

            <div className="flex items-center justify-center gap-4">
              <div className="flex items-center space-x-2">
                <Switch
                  id="batch-mode"
                  checked={isSwitchOn}
                  onCheckedChange={handleSwitchChange}
                  className="data-[state=checked]:bg-blue-600 dark:data-[state=checked]:bg-blue-500"
                />
                <Label htmlFor="batch-mode" className="text-muted-foreground">
                  Batch Mode
                </Label>
              </div>
              <Button size="sm" className="rounded-lg" onClick={handleSelfStake}>
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
                    'relative rounded-lg border border-border/40 dark:bg-zinc-800/90 overflow-hidden select-none',
                    utxo.isSelected
                      ? 'border-blue-400 dark:border-blue-500 bg-blue-50 dark:bg-blue-800/60'
                      : '',
                    isSwitchOn && utxo.isStaked ? 'opacity-50' : 'opacity-100',
                    utxo.isStaked ? 'opacity-50 dark:bg-zinc-900' : '',
                  )}
                >
                  {utxo.isStaked && (
                    <div className="absolute top-0 left-0 px-5 py-0.5 bg-gradient-to-r bg-clip-padding from-teal-500 via-purple-500 to-orange-500 text-white text-xs font-semibold transform -rotate-45 -translate-x-6 translate-y-2">
                      Staked
                    </div>
                  )}
                  <CardHeader className="flex items-center justify-center">
                    <h3 className="text-2xl">UTXO #{utxo.id}</h3>
                  </CardHeader>
                  <CardContent className="flex items-center justify-center">
                    Amount {utxo.amount}
                  </CardContent>
                  {isSwitchOn && !utxo.isStaked && (
                    <div className="absolute top-2 right-2">
                      <CheckCircle2
                        className={cn(
                          'w-5 h-5 text-muted-foreground',
                          utxo.isSelected ? 'text-blue-400' : '',
                        )}
                      />
                    </div>
                  )}
                </Card>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
