import { Routes, Route } from 'react-router-dom'
import { UserAssetsLayout } from '../pages/userAssets/user-assets-layout'
import { UserTransactionsLayout } from '../pages/userTransactions/user-transactions-layout'
import { UserMintLayout } from '../pages/userMint/user-mint-layout'
import { UserAppsLayout } from '../pages/userApps/user-apps-layout'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { UserSettingsLayout } from '../pages/userSettings/user-settings-layout'
import { SftDetailLayout } from '../pages/userMint/sftDetail/sft-detail-layout'
import { Foot } from '@/components/foot'
import { SftDetailLayoutForSelfStaking } from '../pages/userMint/sftDetailForSelfStaking/sft-detail-layout-for-self-staking'

export const MainContent = () => {
  return (
    <div className="flex flex-col h-full bg-background/95">
      <ScrollArea className="w-full whitespace-nowrap flex-grow">
        <ScrollBar orientation="horizontal" />
        <div className="h-full w-full p-4 md:p-6">
          <Routes>
            <Route path="/" element={<UserAssetsLayout />} />
            <Route path="/mint" element={<UserMintLayout />} />
            <Route path="/mint/sft/:sftId" element={<SftDetailLayout />} />
            <Route
              path="/mint/sft/self-staking/:sftId"
              element={<SftDetailLayoutForSelfStaking />}
            />
            <Route path="/apps" element={<UserAppsLayout />} />
            <Route path="/transactions" element={<UserTransactionsLayout />} />
            <Route path="/settings" element={<UserSettingsLayout />} />
          </Routes>
        </div>
      </ScrollArea>
      <Foot />
    </div>
  )
}
