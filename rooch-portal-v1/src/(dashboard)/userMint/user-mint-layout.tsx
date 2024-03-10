import { SftTabs } from './components/sft-tabs'

export const UserMintLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4">
      <div className="flex items-center justify-between space-y-2">
        <span>
          <h1 className="text-3xl font-bold tracking-tight">Mint SFT</h1>
          <p className="text-muted-foreground text-wrap">
            Lorem ipsum dolor sit amet consectetur adipisicing elit. Cum, suscipit?
          </p>
        </span>
      </div>
      <SftTabs />
    </div>
  )
}
