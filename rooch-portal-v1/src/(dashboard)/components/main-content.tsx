import { Routes, Route } from 'react-router-dom'
import { UserAssetsLayout } from '../userAssets/user-assets-layout'
import { UserTransactionsLayout } from '../userTransactions/user-transactions-layout'
import { UserMintLayout } from '../userMint/user-mint-layout'
import { UserAppsLayout } from '../userApps/user-apps-layout'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { UserSettingsLayout } from '../userSettings/user-settings-layout'
import { SftDetailLayout } from '../userMint/sftDetail/sft-detail-layout'
import { Foot } from '@/components/foot'

export const MainContent = () => {
  return (
    <div className="flex flex-col h-full">
      <ScrollArea className="w-full whitespace-nowrap flex-grow">
        <ScrollBar orientation="horizontal" />
        <div className="h-full w-full p-4 md:p-6">
          <Routes>
            <Route path="/" element={<UserAssetsLayout />} />
            <Route path="/mint" element={<UserMintLayout />} />
            <Route path="/mint/sft/:sftId" element={<SftDetailLayout />} />
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
