import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/card'
import { SftCardButtons } from './sft-card-buttons'
import { SftCardContents } from './sft-card-contents'
import { SftCardHeader } from './sft-card-header'

export const SftCard = () => {
  const cards = [1, 2, 3, 4, 5, 6, 7, 8]

  return (
    <>
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
    </>
  )
}
