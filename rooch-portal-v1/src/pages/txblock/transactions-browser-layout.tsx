import { TabItem } from '@/common/interface'

import { TransactionsBrowserTab } from './components/transactions-browser-tab'
import { TransactionsBrowserHeader } from './components/transactions-browser-header'

const tabItems: TabItem[] = [
  { id: 'overview', label: 'Overview', path: '/trasactions/txblock/:hash/' },
  { id: 'userSignatures', label: 'User Signatures', path: '/trasactions/txblock/:hash/' },
  { id: 'rawJson', label: 'Raw Json', path: '/trasactions/txblock/:hash/' },
]

export const TxblockLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-4 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Transaction Header */}
      <TransactionsBrowserHeader />

      {/* Transaction Bar */}
      <TransactionsBrowserTab items={tabItems} />

      {/* Trasaction Details */}
      <div className="rounded-2xl bg-accent h-full w-full">
        <div className="m-2 rounded-2xl bg-background/95 h-full w-full p-4">
          <div>Check Point</div>
        </div>
      </div>
    </div>
  )
}
