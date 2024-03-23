import { useState } from 'react'
import { SupportChain, SupportChains } from '@roochnetwork/rooch-sdk-kit'

import { Button } from '@/components/ui/button'

export const ChainToggle = () => {
  const [chain] = useState<SupportChain>(SupportChains[0])

  const ChainIcon = () => {
    const iconMap: Record<SupportChain, string> = {
      [SupportChain.BITCOIN]: '/icon-btc.svg',
      [SupportChain.ETH]: '/icon-eth.svg',
    }
    return (
      <img
        src={iconMap[chain] || '/icon-default.svg'}
        alt={`${chain} icon`}
        className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 mr-2 hidden md:block"
      />
    )
  }

  return (
    <div className="flex">
      <Button
        variant="ghost"
        size="sm"
        className="cursor-default flex items-center justify-center text-muted-foreground hover:bg-inherit"
      >
        <ChainIcon />
        <h3 className="uppercase">Chain-</h3>
        <h3 className="uppercase">{chain}</h3>
      </Button>
    </div>
  )
}
