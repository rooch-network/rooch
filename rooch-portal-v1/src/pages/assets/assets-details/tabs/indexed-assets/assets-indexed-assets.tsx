import { IndexedAssetsBTC } from './components/indexed-assets-btc'
import { IndexedAssetsOrdi } from './components/indexed-assets-ordi'

export const AssetsIndexedAssets = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg">
      {/* BTC @Bitcoin */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-2">
          <h1 className="text-2xl font-semibold tracking-tight text-primary">BTC @Bitcoin</h1>
        </div>
        <IndexedAssetsBTC />
      </div>

      {/* Ordi @Bitcoin */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-2">
          <h1 className="text-2xl font-semibold tracking-tight text-primary">Ordi @Bitcoin</h1>
        </div>
        <IndexedAssetsOrdi />
      </div>
    </div>
  )
}
