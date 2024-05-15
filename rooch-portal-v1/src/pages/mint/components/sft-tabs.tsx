// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState } from 'react'
import { FeaturedSfts } from './featured-sfts'
import { Tokens } from './tokens'
import { TabItem } from '@/common/interface'
import { AlertCircle } from 'lucide-react'

const mintTabItems: TabItem[] = [
  { id: 'allTokens', label: 'All Tokens', available: false },
  { id: 'featuredTokens', label: 'Featured Tokens', available: false },
]

export const SftTabs = () => {
  const [activeId, setActiveId] = useState<string>('allTokens')

  const handleTabClick = (id: string, available: boolean) => {
    if (available) {
      setActiveId(id)
    }
  }

  const renderComingSoon = () => (
    <div className="flex flex-col items-center justify-center text-center text-xl text-muted-foreground mt-10 animate-pulse">
      <AlertCircle className="w-12 h-12 mb-4 text-blue-500" />
      <p className="mb-2 font-semibold">Coming Soon!</p>
      <p className="text-base text-gray-500">
        We're working hard to bring this feature to you. Stay tuned!
      </p>
    </div>
  )

  const renderTabContent = () => {
    const activeTab = mintTabItems.find((item) => item.id === activeId)
    if (activeTab && !activeTab.available) {
      return renderComingSoon()
    }

    switch (activeId) {
      case 'allTokens':
        return <FeaturedSfts />
      case 'featuredTokens':
        return <Tokens />
      default:
        return <FeaturedSfts />
    }
  }

  return (
    <div>
      <nav className="flex space-x-4 border-b border-accent dark:border-accent/75">
        {mintTabItems.map((item) => (
          <button
            key={item.id}
            className={`px-3 py-2 ${
              activeId === item.id
                ? 'border-b-2 border-blue-500 text-blue-500'
                : 'border-b-2 border-transparent text-muted-foreground'
            } hover:text-blue-500 transition-all`}
            onClick={() => handleTabClick(item.id, item.available)}
          >
            <p className="font-semibold text-sm">{item.label}</p>
          </button>
        ))}
      </nav>

      <div className="mt-4">{renderTabContent()}</div>
    </div>
  )
}
