import { NftCard } from './nft-card'

export const AssetsNft = () => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 w-full place-items-center">
      <NftCard />
    </div>
  )
}
