import { NoData } from '@/components/no-data'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { useCurrentAccount, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

// test address
const testAddress = ''

export const IndexedAssetsBTC = () => {
  const account = useCurrentAccount()

  // TODO: 1, loading, 2 pagination
  const { data: result } = useRoochClientQuery('queryUTXOs', {
    filter: {
      owner: 'bcrt1p79ruqzh9hmmhvaz7x3up3t6pdrmz5hmhz3pfkddxqnfzg0md7upq3jjjev',
    },
  })

  return !result || result.data.length === 0 ? (
    <NoData />
  ) : (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
      {result?.data.map((data) => (
        <Card
          key={data.object_id}
          className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden"
        >
          <CardHeader className="flex items-center justify-center">
            <h3 className="text-xl md:text-2xl">UTXO #{data.tx_order}</h3>
          </CardHeader>
          <CardContent className="flex items-center justify-center text-sm md:text-base">
            {/*Amount {data.amount.toLocaleString()}*/}
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
