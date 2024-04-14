import React, { useEffect, useState } from 'react'
import { useNavigate, useLocation, useParams } from 'react-router-dom'

import { TabItem } from '@/common/interface'

type TabProps = {
  items: TabItem[]
}

export const TransactionsBrowserTab: React.FC<TabProps> = ({ items }) => {
  const navigate = useNavigate()
  const location = useLocation()
  const { hash } = useParams()
  const searchParams = new URLSearchParams(location.search)
  const activeTabFromUrl = searchParams.get('tab')
  const [activeId, setActiveId] = useState<string>(activeTabFromUrl || items[0]?.id || '')

  useEffect(() => {
    setActiveId(activeTabFromUrl || items[0]?.id || '')
  }, [activeTabFromUrl, items])

  const handleTabClick = (id: string) => {
    navigate(`/transactions/txblock/${hash}/?tab=${id}`)
  }

  return (
    <nav className="flex space-x-4 border-b border-accent dark:border-accent/75">
      {items.map((item) => (
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
  )
}
