import { TabItem, TransactionCheckpoint } from '@/common/interface'
import TransactionInfoCard from './transactions-info-card'

type TransactionDetailsProps = {
  tabItems: TabItem[]
  activeTabId: string
}

const transactionInfo: TransactionCheckpoint = {
  checkpoint: 23691432,
  timestamp: '3 mons ago (Jan 16, 2024 08:16:42 +UTC)',
  transactionAction: '0x9b18...65cb',
  action: 'Send',
  amount: 0.591993272,
  currency: 'SUI',
  sender: '0x9b1886b1c9e6107afbb10a4d2a01dbe318776b82021b879007631496919365cb',
  recipients: '0x26fda2e1b4525fa4de9e576156cd184c02e4414f4d33afe3c168698911784cfa',
  status: 'Success',
  totalGasFee: 0.00174788,
  computationFee: 0.00075,
  storageFee: 0.001976,
  storageRebate: 0.00097812,
  gasPayment: '0x006ddc8eded93af1bf87c255b82e3951a7838f9a2017eccb57a0c5bf1b345e54',
  gasBudget: 0.00249788,
  gasPrice: 0.00000075,
}

export const TransactionDetails: React.FC<TransactionDetailsProps> = ({
  tabItems,
  activeTabId,
}) => {
  const activeTab = tabItems.find((item) => item.id === activeTabId)

  console.log(activeTab)

  return (
    <div className="p-4 bg-inherit rounded-lg">
      <TransactionInfoCard transaction={transactionInfo} />
    </div>
  )
}
