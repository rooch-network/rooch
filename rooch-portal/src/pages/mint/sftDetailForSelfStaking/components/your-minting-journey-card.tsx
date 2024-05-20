// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export const YourMintingJourneyCard = () => {
  return (
    <div className="w-full h-full">
      <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
        <CardHeader className="dark:text-zinc-100">
          <CardTitle>Your Minting Journey</CardTitle>
          <CardDescription className="dark:text-zinc-50/70">
            Deploy your new project in one-click.
          </CardDescription>
        </CardHeader>
        <CardContent className="text-sm dark:text-primary grid md:grid-cols-1 gap-6 w-full px-6 md:px-0">
          <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-zinc-50">
            <span className="text-4xl font-medium md:text-5xl">12,333</span>
            <span className="text-base md:text-sm">Distribution Type</span>
          </div>
          <div className="flex flex-row-reverse md:flex-col items-center justify-between md:justify-center gap-2 w-full md:h-28 dark:text-zinc-50">
            <span className="text-4xl font-medium md:text-5xl">12,333</span>
            <span className="text-base md:text-sm">Distribution Type</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
