import { useState } from 'react'

import { FeaturedSfts } from './featured-sfts'
import { Tokens } from './tokens'

import { TabItem } from '@/common/interface'

const mintTabItems: TabItem[] = [
  { id: 'allTokens', label: 'All Tokens' },
  { id: 'featuredTokens', label: 'Featured Tokens' },
]

export const SftTabs = () => {
  const [activeId, setActiveId] = useState<string>('allTokens')

  const handleTabClick = (id: string) => {
    setActiveId(id)
  }

  const renderTabContent = () => {
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
            onClick={() => handleTabClick(item.id)}
          >
            <p className="font-semibold text-sm">{item.label}</p>
          </button>
        ))}
      </nav>

      <div className="mt-4">{renderTabContent()}</div>
    </div>
  )
}
