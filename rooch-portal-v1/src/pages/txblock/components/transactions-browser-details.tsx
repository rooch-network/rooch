import React from 'react'

import { TabItem } from '@/common/interface'

import { Overview } from '../tabs/Overview'
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
        return <Overview />
      case 'userSignatures':
        return <UserSignatures />
      case 'rawJson':
        return <RawJson />
      default:
        return <Overview />
    }
  }

  return <div>{renderContent()}</div>
}
