import { NoData } from '@/components/no-data'
import { Card, CardContent, CardHeader } from '@/components/ui/card'

interface MockData {
  id: number
  amount: number
}

const mockData: MockData[] = [
  { id: 123, amount: 1234 },
  { id: 456, amount: 2468 },
  { id: 789, amount: 3690 },
  { id: 1011, amount: 4812 },
]

export const IndexedAssetsBTC = () => {
  if (mockData.length === 0) {
    return <NoData />
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
      {mockData.map((data) => (
        <Card
          key={data.id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
        >
          <CardHeader className="flex items-center justify-center">
            <h3 className="text-xl md:text-2xl">UTXO #{data.id}</h3>
          </CardHeader>
          <CardContent className="flex items-center justify-center text-sm md:text-base">
            Amount {data.amount.toLocaleString()}
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
