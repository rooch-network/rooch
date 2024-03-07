import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { AssetsCoin } from './assets-coin'
import { AssetsNft } from './assets-nft'
import { AssetsSft } from './assets-sft'

export const AssetsTabs = () => {
  return (
    <Tabs defaultValue="coin">
      <TabsList className="grid grid-cols-3 w-full transition-all">
        <TabsTrigger value="coin">🪙 Coin</TabsTrigger>
        <TabsTrigger value="nft">🎆 NFT</TabsTrigger>
        <TabsTrigger value="sft">🎇 SFT</TabsTrigger>
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
