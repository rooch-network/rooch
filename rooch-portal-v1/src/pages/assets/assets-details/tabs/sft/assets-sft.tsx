import { SftCard } from './components/sft-card'
// import { SftTabHeader } from './components/sft-tab-header'

export const AssetsSft = () => {
  return (
    <>
      {/* <div>
        <SftTabHeader />
      </div> */}
      <div className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-4 w-full place-items-start mt-2">
        <SftCard />
      </div>
    </>
  )
}
