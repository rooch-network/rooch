import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { FeaturedSfts } from './featured-sfts'
import { Tokens } from './tokens'

export const SftTabs = () => {
  return (
    <Tabs defaultValue="featured">
      <TabsList className="grid w-full grid-cols-2 md:w-[400px]">
        <TabsTrigger value="featured">Featured Tokens</TabsTrigger>
        <TabsTrigger value="tokens">All Tokens</TabsTrigger>
      </TabsList>
      <TabsContent value="featured">
        {/* Featured SFTs */}
        <FeaturedSfts />
      </TabsContent>
      <TabsContent value="tokens">
        {/* Tokens */}
        <Tokens />
      </TabsContent>
    </Tabs>
  )
}
