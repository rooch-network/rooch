import { TabItem } from '@/common/interface'
import React, { useState } from 'react'

type TabProps = {
  items: TabItem[]
}

export const TransactionsBrowserTab: React.FC<TabProps> = ({ items }) => {
  const [activeId, setActiveId] = useState<string>('')

  return (
    <nav className="flex space-x-4">
      {items.map((item) => (
        <button
          key={item.id}
          className={`px-3 py-2 ${
            activeId === item.id ? 'border-b-2 border-blue-500' : 'border-b-2 border-transparent'
          } hover:border-blue-500 transition-colors duration-300`}
          onClick={() => setActiveId(item.id)}
        >
          {item.label}
        </button>
      ))}
    </nav>
  )
}
