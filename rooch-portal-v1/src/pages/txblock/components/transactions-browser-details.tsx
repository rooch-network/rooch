import { TabItem } from '@/common/interface'
import TransactionInfoCard from './transactions-info-card'

type TransactionDetailsProps = {
  tabItems: TabItem[]
  activeTabId: string
}

export const TransactionDetails: React.FC<TransactionDetailsProps> = ({
  tabItems,
  activeTabId,
}) => {
  const activeTab = tabItems.find((item) => item.id === activeTabId)

  console.log(activeTab)

  return (
    <div>
      <TransactionInfoCard />
    </div>
  )
}
