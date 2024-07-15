// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { ReactNode } from 'react'
import { useNavigate, useParams } from 'react-router-dom'

import { TransactionWithInfoView } from '@roochnetwork/rooch-sdk'

import { TabItem } from '@/common/interface'
import { TabView } from '@/view/tab-view'
import { RawJson } from '../tabs/raw-json'
import { Overview } from '../tabs/overview'

type TabProps = {
  data?: TransactionWithInfoView
}

const tabItems: TabItem[] = [
  { id: 'overview', label: 'Overview', available: true },
  { id: 'rawJson', label: 'Raw Json', available: true },
]

export const TransactionsTabLayout: React.FC<TabProps> = ({ data }) => {
  const navigate = useNavigate()
  const { hash } = useParams()

  const handleTabClick = (id: string) => {
    navigate(`/transactions/detail/${hash}/?tab=${id}`)
  }

  const renderContent = (id: string): ReactNode => {
    let content: ReactNode
    switch (id) {
      case 'rawJson' :
        content = <RawJson txData={data} />
        break
      default:
        content = <Overview txData={data} />
        break
    }

    return <div className="rounded-2xl bg-accent/75 h-full w-full">
      <div className="m-3 rounded-2xl bg-background/95 h-full p-4 shadow-sm">
        {content}
      </div>
    </div>
  }

  return (
    <TabView items={tabItems} tabClick={handleTabClick} renderContent={renderContent} />
  )
}
