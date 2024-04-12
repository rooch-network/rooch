import React from 'react'
import { TabItem } from '@/common/interface'
import TransactionInfoCard from './transactions-info-card'
import { UserSignatures } from '../tabs/user-signatures'
import { RawJson } from '../tabs/raw-json'

type TransactionDetailsProps = {
  tabItems: TabItem[]
  activeTabId: string
}

export const TransactionDetails: React.FC<TransactionDetailsProps> = ({ activeTabId }) => {
  const renderContent = () => {
    switch (activeTabId) {
      case 'overview':
        return <TransactionInfoCard />
      case 'userSignatures':
        return <UserSignatures />
      case 'rawJson':
        return <RawJson />
      default:
        return <TransactionInfoCard />
    }
  }

  return <div>{renderContent()}</div>
}
