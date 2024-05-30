// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons'

import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
} from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useState } from 'react'

const sfts = [
  {
    value: 'rooch',
    label: 'Rooch',
  },
  {
    value: 'rordi',
    label: 'ROrdi',
  },
  {
    value: 'rxai',
    label: 'RXai',
  },
  {
    value: 'rstrk',
    label: 'RStrk',
  },
  {
    value: 'rdoge',
    label: 'RDoge',
  },
]

export const SftTabHeader = () => {
  const [open, setOpen] = useState(false)
  const [value, setValue] = useState('')
  const [isSwitchOn, setIsSwitchOn] = useState(false)

  const handleSwitchChange = (checked: boolean) => {
    setIsSwitchOn(checked)
  }

  return (
    <div className="flex items-center justify-start gap-x-3">
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            size="sm"
            className="w-full md:w-[150px] justify-between rounded-lg"
          >
            {value ? sfts.find((sft) => sft.value === value)?.label : 'Select sft...'}
            <CaretSortIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-max md:w-[150px] p-0">
          <Command>
            <CommandInput placeholder="Search SFT..." className="h-9" />
            <CommandEmpty>No SFT found.</CommandEmpty>
            <CommandGroup>
              {sfts.map((sft) => (
                <CommandItem
                  key={sft.value}
                  value={sft.value}
                  onSelect={(currentValue) => {
                    setValue(currentValue === value ? '' : currentValue)
                    setOpen(false)
                  }}
                >
                  {sft.label}
                  <CheckIcon
                    className={cn(
                      'ml-auto h-4 w-4',
                      value === sft.value ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </Command>
        </PopoverContent>
      </Popover>
      <div className="flex items-center space-x-2">
        <Switch
          id="batch-mode"
          checked={isSwitchOn}
          onCheckedChange={handleSwitchChange}
          className="data-[state=checked]:bg-blue-600 dark:data-[state=checked]:bg-blue-500"
        />
        <Label htmlFor="batch-mode">Batch Mode</Label>
      </div>
    </div>
  )
}
