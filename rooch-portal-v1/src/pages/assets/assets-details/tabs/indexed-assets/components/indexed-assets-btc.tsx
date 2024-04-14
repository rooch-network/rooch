import { Card, CardContent, CardHeader } from '@/components/ui/card'

export const IndexedAssetsBTC = () => {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
      <Card className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden">
        <CardHeader className="flex items-center justify-center">
          <h3 className="text-xl md:text-2xl">UTXO #123</h3>
        </CardHeader>
        <CardContent className="flex items-center justify-center text-sm md:text-base">
          Amount 1,234
        </CardContent>
      </Card>
      <Card className="w-full transition-all border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden">
        <CardHeader className="flex items-center justify-center">
          <h3 className="text-xl md:text-2xl">UTXO #456</h3>
        </CardHeader>
        <CardContent className="flex items-center justify-center text-sm md:text-base">
          Amount 2,468
        </CardContent>
      </Card>
    </div>
  )
}
