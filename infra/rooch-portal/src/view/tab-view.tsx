// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useCurrentWallet } from '@roochnetwork/rooch-sdk-kit'

import { TabItem } from '@/common/interface'
import { ConnectWalletHint } from '@/components/connect-wallet-hint'
import { useNavigate, useLocation } from 'react-router-dom'
import { ReactNode } from 'react'

type TabViewProps = {
  items: TabItem[]
  tabClick? : (id: string) => void
  renderContent?: (id: string) => ReactNode
}

export const TabView: React.FC<TabViewProps> = ({items, tabClick, renderContent}) => {
  const navigate = useNavigate()
  const location = useLocation()
  const searchParams = new URLSearchParams(location.search)
  const activeTabId = searchParams.get('tab') || items[0].id
  const { isConnected } = useCurrentWallet()

  const handleTabClick = (id: string) => {
    navigate(`${location.pathname}?tab=${id}`)
  }

  const renderTabContent = () => {
    if (!isConnected) {
      return <ConnectWalletHint/>
    }

    if (renderContent) {
      return renderContent(activeTabId)
    }

    const find =  items.find((item) => item.id === activeTabId && item.children !== undefined)

    return find? find.children : <></>
  }

  return (
    <div>
      <nav className="flex space-x-4 border-b border-accent dark:border-accent/75">
        {items.map((item) => (
          <button
            key={item.id}
            className={`px-3 py-2 ${
              activeTabId === item.id
                ? 'border-b-2 border-blue-500 text-blue-500'
                : 'border-b-2 border-transparent text-muted-foreground'
            } hover:text-blue-500 transition-all`}
            onClick={() => tabClick ? tabClick(item.id) : handleTabClick(item.id)}
          >
            <p className="font-semibold text-sm">{item.label}</p>
          </button>
        ))}
      </nav>

      <div className="mt-4">{renderTabContent()}</div>
    </div>
  )
}
