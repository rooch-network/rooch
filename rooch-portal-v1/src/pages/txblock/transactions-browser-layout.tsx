import { TabItem } from '@/common/interface'

import { TransactionsBrowserTab } from './components/transactions-browser-tab'
import { TransactionsBrowserHeader } from './components/transactions-browser-header'
import { TransactionDetails } from './components/transactions-browser-details'

const tabItems: TabItem[] = [
  { id: 'overview', label: 'Overview', path: '/trasactions/txblock/:hash/' },
  { id: 'userSignatures', label: 'User Signatures', path: '/trasactions/txblock/:hash/' },
  { id: 'rawJson', label: 'Raw Json', path: '/trasactions/txblock/:hash/' },
]

export const TransactionsBrowserLayout = () => {
  const activeTabId = tabItems[0].id

  return (
    <div className="h-full flex-1 flex-col space-y-4 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Transaction Header */}
      <TransactionsBrowserHeader />

      {/* Transaction Bar */}
      <TransactionsBrowserTab items={tabItems} />

      {/* Trasaction Details */}
      <div className="rounded-2xl bg-accent/75 h-full w-full">
        <div className="m-3 rounded-2xl bg-background/95 h-full p-4 shadow-sm">
          <TransactionDetails tabItems={tabItems} activeTabId={activeTabId} />
        </div>
      </div>
    </div>
  )
}
