import { IndexedAssetsBTC } from './indexed-assets-btc'
import { IndexedAssetsOrdi } from './indexed-assets-ordi'

export const AssetsIndexedAssets = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg">
      {/* BTC @Bitcoin */}
      <div>
        <div className="flex items-center justify-between space-y-2">
          <h1 className="text-2xl font-bold tracking-tight text-primary/80">BTC @Bitcoin</h1>
        </div>
        <IndexedAssetsBTC />
      </div>

      {/* Ordi @Bitcoin */}
      <div>
        <div className="flex items-center justify-between space-y-2">
          <h1 className="text-2xl font-bold tracking-tight text-primary/80">Ordi @Bitcoin</h1>
        </div>
        <IndexedAssetsOrdi />
      </div>
    </div>
  )
}
