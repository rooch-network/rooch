import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { AssetsCoin } from './assets-coin'
import { AssetsNft } from './assets-nft'
import { AssetsSft } from './assets-sft'
import { AssetsIndexedAssets } from './assets-indexed-assets'

export const AssetsTabs = () => {
  return (
    <Tabs defaultValue="coin">
      <TabsList className="grid grid-cols-4 transition-al md:h-10 overflow-auto">
        <TabsTrigger value="coin" className="h- min-w-max md:w-full">
          <div className="flex items-center justify-center">
            <img src="/icon-coin.svg" alt="Coin" className="h-[1.2rem] w-[1.2rem] mr-1" />
            <span className="font-semibold text-sm">Coin</span>
          </div>
        </TabsTrigger>
        <TabsTrigger value="nft" className="h-full min-w-max md:w-full">
          <div className="flex items-center justify-center">
            <img src="/icon-nft.svg" alt="NFT" className="h-[1.2rem] w-[1.2rem] mr-1" />
            <span className="font-semibold text-sm">NFT</span>
          </div>
        </TabsTrigger>
        <TabsTrigger value="sft" className="h-full min-w-max md:w-full">
          <div className="flex items-center justify-center">
            <img src="/icon-sft.svg" alt="SFT" className="h-[1.2rem] w-[1.2rem] mr-1" />
            <span className="font-semibold text-sm">SFT</span>
          </div>
        </TabsTrigger>
        <TabsTrigger value="indexed_assets" className="h-full min-w-max md:w-full">
          <div className="flex items-center justify-center">
            <img
              src="/icon-index-assets.svg"
              alt="Index Assets"
              className="h-[1.2rem] w-[1.2rem] mr-1"
            />
            <span className="font-semibold text-sm">Indexed Assets</span>
          </div>
        </TabsTrigger>
      </TabsList>
      <TabsContent value="coin">
        <AssetsCoin />
      </TabsContent>
      <TabsContent value="nft">
        <AssetsNft />
      </TabsContent>
      <TabsContent value="sft">
        <AssetsSft />
      </TabsContent>
      <TabsContent value="indexed_assets">
        <AssetsIndexedAssets />
      </TabsContent>
    </Tabs>
  )
}
