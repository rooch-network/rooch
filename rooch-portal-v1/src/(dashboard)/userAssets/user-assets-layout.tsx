import { AssetsTabs } from './components/assets-tabs'
import { ProfileCard } from './components/profile-card'

export const UserAssetsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4">
      <ProfileCard />
      <AssetsTabs />
    </div>
  )
}
