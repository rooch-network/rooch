// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { BitcoinAssetsBtc } from './components/bitcoin-assets-btc.tsx'
import { BitcoinAssetsOrdi } from './components/bitcoin-assets-ordi.tsx'
import { Badge } from '@/components/ui/badge.tsx'

export const AssetsBitcoinAssets = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg">
      {/* BTC @Bitcoin */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-2">
          <h1 className="text-2xl font-semibold tracking-tight text-primary">BTC</h1>
        </div>
        <BitcoinAssetsBtc />
      </div>

      {/* Ordi @Bitcoin */}
      <div>
        <div className="flex items-center justify-start mb-2 gap-2">
          <h1 className="text-2xl font-semibold tracking-tight text-primary">Ordi</h1>
          <Badge
            variant="outline"
            className="rounded-full border-amber-500 text-amber-500 dark:border-amber-300 dark:text-amber-300 hover:bg-amber-500/10"
          >
            Ordinals Protocol
          </Badge>
        </div>
        <BitcoinAssetsOrdi />
      </div>
    </div>
  )
}
