import { FC, useState } from 'react'

import { TESTNetwork, Network } from '@roochnetwork/rooch-sdk'
import { NetworkIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'

export const NetworkToggle: FC = () => {
  const [network] = useState<Network>(TESTNetwork)

  return (
    <div className="hidden md:flex">
      <Button
        variant="ghost"
        size="sm"
        className="cursor-default flex items-center justify-center hover:bg-inherit"
      >
        <NetworkIcon className="h-[1rem] w-[1rem] mr-1" />
        <h3 className="uppercase">{network.name}</h3>
      </Button>
    </div>
  )
}
