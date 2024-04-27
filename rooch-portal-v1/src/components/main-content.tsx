import { Foot } from '@/components/foot'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { routers } from '@/navigation'

export const MainContent = () => {
  return (
    <div className="flex flex-col h-full bg-background/95">
      <ScrollArea className="w-full whitespace-nowrap flex-grow">
        <ScrollBar orientation="horizontal" />
        <div className="h-full w-full p-4 md:p-6">{routers()}</div>
      </ScrollArea>
      <Foot />
    </div>
  )
}
