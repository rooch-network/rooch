// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react'

import { TabItem } from '@/common/interface'

import { Overview } from '../tabs/Overview'
import { RawJson } from '../tabs/raw-json'
import { TransactionWithInfoView } from '@roochnetwork/rooch-sdk'

type TransactionDetailsProps = {
  txData: TransactionWithInfoView
  tabItems: TabItem[]
  activeTabId: string
}

export const TransactionDetails: React.FC<TransactionDetailsProps> = ({ activeTabId, txData }) => {
  const renderContent = () => {
    switch (activeTabId) {
      case 'overview':
        return <Overview txData={txData} />
      case 'rawJson':
        return <RawJson txData={txData} />
      default:
        return <Overview txData={txData} />
    }
  }

  return <>{renderContent()}</>
}
