import { useState } from 'react'
import { TabItem } from '@/common/interface'
import { AssetsCoin } from './tabs/coin/assets-coin'
import { AssetsNft } from './tabs/nft/assets-nft'
import { AssetsSft } from './tabs/sft/assets-sft'
import { AssetsIndexedAssets } from './tabs/indexed-assets/assets-indexed-assets'

const assetsTabItems: TabItem[] = [
  { id: 'coin', label: 'Coin' },
  { id: 'nft', label: 'NFT' },
  { id: 'sft', label: 'SFT' },
  { id: 'indexed_assets', label: 'Indexed Assets' },
]

export const AssetsDetails = () => {
  const [activeId, setActiveId] = useState<string>('coin')

  const handleTabClick = (id: string) => {
    setActiveId(id)
  }

  const renderTabContent = () => {
    switch (activeId) {
      case 'coin':
        return <AssetsCoin />
      case 'nft':
        return <AssetsNft />
      case 'sft':
        return <AssetsSft />
      case 'indexed_assets':
        return <AssetsIndexedAssets />
      default:
        return <AssetsCoin />
    }
  }

  return (
    <div>
      <nav className="flex space-x-4 border-b border-accent dark:border-accent/75">
        {assetsTabItems.map((item) => (
          <button
            key={item.id}
            className={`px-3 py-2 text-muted-foreground ${
              activeId === item.id
                ? 'border-b-2 border-blue-500 text-blue-500'
                : 'border-b-2 border-transparent'
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
