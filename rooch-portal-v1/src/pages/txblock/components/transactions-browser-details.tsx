import React from 'react'

import { TabItem } from '@/common/interface'

import { Overview } from '../tabs/Overview'
import { UserSignatures } from '../tabs/user-signatures'
import { RawJson } from '../tabs/raw-json'
import {TransactionWithInfoView} from '@roochnetwork/rooch-sdk';

type TransactionDetailsProps = {
  txData: TransactionWithInfoView
  tabItems: TabItem[]
  activeTabId: string
}

export const TransactionDetails: React.FC<TransactionDetailsProps> = ({ activeTabId, txData }) => {
  const renderContent = () => {
    switch (activeTabId) {
      case 'overview':
        return <Overview txData={txData}/>
      case 'userSignatures':
        return <UserSignatures seqData={txData.transaction.sequence_info}/>
      case 'rawJson':
        return <RawJson txData={txData}/>
      default:
        return <Overview txData={txData}/>
    }
  }

  return <>{renderContent()}</>
}
