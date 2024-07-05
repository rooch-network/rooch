// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/card'
import { SftCardHeader } from './components/sft-card-header'
import { SftCardContents } from './components/sft-card-contents'
import { SftCardButtons } from './components/sft-card-buttons'
import { NoData } from '@/components/no-data'

export const AssetsSft = () => {
  const cards = [1, 2, 3, 4, 5, 6, 7, 8]

  if (cards.length === 0) {
    return <NoData />
  }

  return (
    <>
      <div className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-4 w-full place-items-start mt-2">
        {cards.map((index) => (
          <Card
            key={index}
            className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
          >
            <CardHeader className="py-4 px-2 md:px-4">
              <SftCardHeader />
            </CardHeader>
            <CardContent className="p-0 flex items-center justify-center h-[80px]">
              <SftCardContents />
            </CardContent>
            <CardFooter className="p-2 md:p-4 flex flex-wrap gap-1 md:flex-row">
              <SftCardButtons />
            </CardFooter>
          </Card>
        ))}
      </div>
    </>
  )
}
