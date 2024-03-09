import { FC, useState } from 'react'

import { TESTNetwork, Network } from '@roochnetwork/rooch-sdk'
import { NetworkIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'

export const NetworkToggle: FC = () => {
  const [network] = useState<Network>(TESTNetwork)

  return (
    <div className="flex">
      <Button
        variant="ghost"
        size="sm"
        className="cursor-default flex items-center justify-center hover:bg-inherit"
      >
        <NetworkIcon className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all md:mr-2" />
        <h3 className="uppercase hidden md:block">{network.name}</h3>
      </Button>
    </div>
  )
}
