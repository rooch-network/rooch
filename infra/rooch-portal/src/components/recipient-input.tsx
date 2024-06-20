// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'

interface RecipientInputProps {
  recipient: string
  onChange: (event: React.ChangeEvent<HTMLTextAreaElement>) => void
  disabled: boolean
}

const RecipientInput: React.FC<RecipientInputProps> = ({ recipient, onChange, disabled }) => {
  return (
    <div className="grid w-full max-w-md items-center gap-1.5">
      <Label htmlFor="address">Send to</Label>
      <Textarea
        id="address"
        placeholder="Enter Address..."
        className="h-14 resize-none overflow-hidden rounded-2xl bg-gray-50 dark:bg-gray-200 text-gray-800 w-96"
        value={recipient}
        onChange={onChange}
        disabled={disabled}
        required
        rows={1}
      />
    </div>
  )
}

export default RecipientInput
