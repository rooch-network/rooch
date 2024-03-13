import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { AssetsCoin } from './assets-coin'
import { AssetsNft } from './assets-nft'
import { AssetsSft } from './assets-sft'

export const AssetsTabs = () => {
  return (
    <Tabs defaultValue="coin">
      <TabsList className="grid grid-cols-3 w-full transition-all h-10">
        <TabsTrigger value="coin" className="h-full">
          <div className="flex items-center justify-center">
            <img src="/icon-coin.svg" alt="Coin" className="h-[1.2rem] w-[1.2rem] mr-1" />
            <span className="font-semibold text-sm">Coin</span>
          </div>
        </TabsTrigger>
        <TabsTrigger value="nft" className="h-full">
          🎆 NFT
        </TabsTrigger>
        <TabsTrigger value="sft" className="h-full">
          🎇 SFT
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
    </Tabs>
  )
}
