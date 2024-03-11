import { useState } from 'react'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

export const ChainToggle = () => {
  const [chain, setChain] = useState('btc')

  const ChainIcon = () => {
    switch (chain) {
      case 'btc':
        return (
          <img
            src="/icon-bitcoin.svg"
            className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all"
          />
        )
      case 'eth':
        return (
          <img
            src="/icon-eth.svg"
            className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all"
          />
        )
      default:
        return (
          <img
            src="/icon-bitcoin.svg"
            className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all"
          />
        )
    }
  }

  const getChainName = () => {
    switch (chain) {
      case 'btc':
        return 'Bitcoin'
      case 'eth':
        return 'Ethereum'
      default:
        return 'Bitcoin'
    }
  }

  const dropdownMenu = () => {
    return (
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>My Chain</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => setChain('btc')}>
          <div className="flex items-center justify-start gap-x-2">Bitcoin</div>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setChain('eth')}>
          <div className="flex items-center justify-start gap-x-2">Ethereum</div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    )
  }

  return (
    <div className="hidden md:flex">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="sm" className="select-none">
            <ChainIcon />
            <span className="ml-2">{getChainName()}</span>
          </Button>
        </DropdownMenuTrigger>
        {dropdownMenu()}
      </DropdownMenu>
    </div>
  )
}
