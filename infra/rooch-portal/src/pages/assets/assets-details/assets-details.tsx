// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { TabItem } from '@/common/interface'
import { AssetsCoin } from './tabs/coin/assets-coin'
import { AssetsNft } from './tabs/nft/assets-nft'
import { AssetsBitcoinAssets } from '@/pages/assets/assets-details/tabs/bitcoin-assets/assets-bitcoin-assets.tsx'

import { TabView } from '@/view/tab-view.tsx'

const assetsTabItems: TabItem[] = [
  { id: 'coin', label: 'Coin', available: true, children: <AssetsCoin /> },
  { id: 'nft', label: 'NFT', available: true, children: <AssetsNft /> },
  {
    id: 'bitcoin_assets',
    label: 'Bitcoin Assets',
    available: true,
    children: <AssetsBitcoinAssets />,
  },
]

export const AssetsDetails = () => {
  return <TabView items={assetsTabItems} />
}
