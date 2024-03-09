import { FC, useState } from 'react'

import { TESTNetwork, Network } from '@roochnetwork/rooch-sdk'
import { NetworkIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'

export const NetworkToggle: FC = () => {
  const [network] = useState<Network>(TESTNetwork)

  return (
    <div className="flex w-full">
      <Button
        variant="ghost"
        size="sm"
        className="cursor-default flex items-center justify-center text-muted-foreground hover:bg-inherit w-full p-0"
      >
        <NetworkIcon className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 md:mr-2 hidden md:block" />
        <h3 className="uppercase">Network-{network.name}</h3>
      </Button>
    </div>
  )
}
