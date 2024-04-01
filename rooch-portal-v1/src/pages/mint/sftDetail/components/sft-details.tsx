import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { useEffect, useState } from 'react'
import { OverviewCard } from './overview-card'
import { YourMintingJourneyCard } from './your-minting-journey-card'

export function SftDetails() {
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    const timer = setTimeout(() => setProgress(84.4), 500)
    return () => clearTimeout(timer)
  }, [])

  return (
    <>
      <div className="flex items-center justify-start w-full gap-2 text-muted-foreground dark:text-teal-50 mt-2">
        <span className="text-sm">Process</span>
        <Progress value={progress} />
        <span className="text-sm flex items-center gap-1">
          <p className="font-semibold">84.4%</p>
          <p>Minted</p>
        </span>
      </div>

      <div className="grid md:grid-cols-2 gap-x-6 w-full mt-6 gap-4 md:gap-6">
        <OverviewCard />
        <YourMintingJourneyCard />
      </div>

      <Button className="rounded-lg w-full mt-4 mb-2 md:mt-8 md:mb-6 h-12 dark:bg-teal-500 dark:hover:bg-teal-400 text-white bg-teal-500 hover:bg-teal-600 font-semibold">
        Mint
      </Button>
    </>
  )
}
