import { useState, FC } from 'react'
import { SupportChain, SupportChains } from '@roochnetwork/rooch-sdk-kit'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

export const ChainToggle: FC = () => {
  const [chain, setChain] = useState<SupportChain>(SupportChains[0])

  const ChainIcon: FC = () => {
    const iconMap: Record<SupportChain, string> = {
      [SupportChain.BITCOIN]: '/icon-bitcoin.svg',
      [SupportChain.ETH]: '/icon-eth.svg',
    }
    return (
      <img
        src={iconMap[chain] || '/icon-default.svg'}
        alt={`${chain} icon`}
        className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all"
      />
    )
  }

  const getChainName = (chainIdentifier: SupportChain): string => {
    const nameMap: Record<SupportChain, string> = {
      [SupportChain.BITCOIN]: 'Bitcoin',
      [SupportChain.ETH]: 'Ethereum',
    }
    return nameMap[chainIdentifier] || 'Unknown'
  }

  const dropdownMenu = () => (
    <DropdownMenuContent align="end">
      <DropdownMenuLabel>My Chain</DropdownMenuLabel>
      <DropdownMenuSeparator />
      {SupportChains.map((supportedChain: SupportChain) => (
        <DropdownMenuItem key={supportedChain} onClick={() => setChain(supportedChain)}>
          <div className="flex items-center justify-start gap-x-2">
            {getChainName(supportedChain)}
          </div>
        </DropdownMenuItem>
      ))}
    </DropdownMenuContent>
  )

  return (
    <div className="hidden md:flex">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="sm" className="select-none">
            <ChainIcon />
            <span className="ml-2">{getChainName(chain)}</span>
          </Button>
        </DropdownMenuTrigger>
        {dropdownMenu()}
      </DropdownMenu>
    </div>
  )
}
