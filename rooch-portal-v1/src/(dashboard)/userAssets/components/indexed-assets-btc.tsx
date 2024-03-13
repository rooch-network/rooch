import { Card, CardContent, CardHeader } from '@/components/ui/card'

export const IndexedAssetsBTC = () => {
  return (
    <div className="grid grid-cols-4 gap-4">
      <Card className="rounded-lg border border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden">
        <CardHeader className="flex items-center justify-center">
          <h3 className="text-2xl">UTXO #123</h3>
        </CardHeader>
        <CardContent className="flex items-center justify-center">Amount 1,234</CardContent>
      </Card>
      <Card className="rounded-lg border border-border/40 dark:bg-zinc-800/90 dark:hover:border-primary/20 hover:shadow-md overflow-hidden">
        <CardHeader className="flex items-center justify-center">
          <h3 className="text-2xl">UTXO #123</h3>
        </CardHeader>
        <CardContent className="flex items-center justify-center">Amount 1,234</CardContent>
      </Card>
    </div>
  )
}
