import { TransactionsTable } from './components/transactions-table'

export const UserTransactionsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      <div className="flex items-center justify-between space-y-2">
        <span>
          <h1 className="text-3xl font-bold tracking-tight">Transactions</h1>
          <p className="text-muted-foreground text-wrap">Browse your transactions history.</p>
        </span>
      </div>
      <TransactionsTable />
    </div>
  )
}
