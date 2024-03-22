import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export const SelfStakingCard = () => {
  return (
    <div className="mt-6">
      <div className="h-full w-full">
        <Card className="h-full border-border/40 shadow-inner bg-border/10 dark:bg-border/60">
          <CardHeader className="dark:text-teal-100">
            <CardTitle>My Bitcoin UTXO</CardTitle>
            <CardDescription className="dark:text-teal-50/70">
              Stake your UTXO below
            </CardDescription>
          </CardHeader>
          <CardContent>
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
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
