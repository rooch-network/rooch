// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { formatCoin } from '@/utils/format'
import { BalanceInfoView } from '@roochnetwork/rooch-sdk'

interface AmountInputProps {
  amount: string
  onChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  selectedCoin: BalanceInfoView | null
  error: string
  disabled: boolean
}

const AmountInput: React.FC<AmountInputProps> = ({
  amount,
  onChange,
  selectedCoin,
  error,
  disabled,
}) => {
  return (
    <div className="grid w-full max-w-md items-center gap-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor="amount">Amount</Label>
        <p className="text-xs text-muted-foreground">
          <span>Balance: </span>
          <span className="font-semibold text-blue-600 dark:text-blue-400">
            {selectedCoin ? formatCoin(Number(selectedCoin.balance), selectedCoin.decimals) : '0.0'}
          </span>
        </p>
      </div>
      <div className="h-14 rounded-2xl bg-zinc-200 dark:bg-zinc-700 w-96 pl-6 flex items-center justify-between relative">
        <div className="flex items-center justify-center gap-1.5">
          <span className="bg-white rounded-full p-0.5">
            <img src="/rooch_black_logo.svg" alt="rooch" className="w-4 h-4" />
          </span>
          <p className="text-sm">{selectedCoin?.name}</p>
        </div>
        <Input
          id="amount"
          className="h-10 rounded-2xl bg-gray-50 dark:bg-gray-200 text-gray-800 w-48 pr-8 mr-2 overflow-hidden border-none"
          placeholder="0.0"
          value={amount}
          onChange={onChange}
          disabled={disabled}
          required
          pattern="[0-9]*\.?[0-9]*"
        />
      </div>
      {error && <p className="text-red-500 text-xs mt-1">{error}</p>}
    </div>
  )
}

export default AmountInput
