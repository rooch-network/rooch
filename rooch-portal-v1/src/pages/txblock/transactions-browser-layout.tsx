import { useLocation, useParams } from 'react-router-dom'

import { TransactionsBrowserTab } from './components/transactions-browser-tab'
import { TransactionsBrowserHeader } from './components/transactions-browser-header'
import { TransactionDetails } from './components/transactions-browser-details'

import { TabItem } from '@/common/interface'
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

const tabItems: TabItem[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'userSignatures', label: 'User Signatures' },
  { id: 'rawJson', label: 'Raw Json' },
]

// TODO: loading
export const TransactionsBrowserLayout = () => {
  const location = useLocation()
  const searchParams = new URLSearchParams(location.search)
  const activeTabId = searchParams.get('tab') || 'overview'
  const { hash } = useParams()

  const { data: result, isPending } = useRoochClientQuery('queryTransactions', {
    filter: {
      tx_hashes: [hash || ''],
    },
  })

  return isPending ? (
    <>loading</>
  ) : (
    <div className="h-full flex-1 flex-col space-y-4 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Transaction Header */}
      <TransactionsBrowserHeader />

      {/* Transaction Bar */}
      <TransactionsBrowserTab items={tabItems} />

      {/* Trasaction Details */}
      <div className="rounded-2xl bg-accent/75 h-full w-full">
        <div className="m-3 rounded-2xl bg-background/95 h-full p-4 shadow-sm">
          <TransactionDetails
            txData={result!.data[0]}
            tabItems={tabItems}
            activeTabId={activeTabId}
          />
        </div>
      </div>
    </div>
  )
}
