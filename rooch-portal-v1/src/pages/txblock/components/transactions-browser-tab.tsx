import React, { useState } from 'react'
import { TabItem } from '@/common/interface'

type TabProps = {
  items: TabItem[]
}

export const TransactionsBrowserTab: React.FC<TabProps> = ({ items }) => {
  const [activeId, setActiveId] = useState<string>(items[0]?.id || '')

  return (
    <nav className="flex space-x-4 border-b border-accent dark:border-accent/75">
      {items.map((item) => (
        <button
          key={item.id}
          className={`px-3 py-2 text-muted-foreground ${
            activeId === item.id
              ? 'border-b-2 border-blue-500 text-blue-500'
              : 'border-b-2 border-transparent'
          } hover:text-blue-500 transition-all`}
          onClick={() => setActiveId(item.id)}
        >
          <p className="font-semibold text-sm">{item.label}</p>
        </button>
      ))}
    </nav>
  )
}
