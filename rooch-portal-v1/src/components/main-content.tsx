import { Routes, Route } from 'react-router-dom'

import { Foot } from '@/components/foot'

import { AppsLayout } from '@/pages/apps/apps-layout'
import { MintLayout } from '@/pages/mint/mint-layout'
import { AssetsLayout } from '@/pages/assets/assets-layout'
import { SettingsLayout } from '@/pages/settings/settings-layout'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { SftDetailLayout } from '@/pages/mint/sftDetail/sft-detail-layout'
import { TransactionsLayout } from '@/pages/transactions/transactions-layout'
import { SftDetailLayoutForSelfStaking } from '@/pages/mint/sftDetailForSelfStaking/sft-detail-layout-for-self-staking'

export const MainContent = () => {
  return (
    <div className="flex flex-col h-full bg-background/95">
      <ScrollArea className="w-full whitespace-nowrap flex-grow">
        <ScrollBar orientation="horizontal" />
        <div className="h-full w-full p-4 md:p-6">
          <Routes>
            <Route path="/" element={<AssetsLayout />} />
            <Route path="/mint" element={<MintLayout />} />
            <Route path="/mint/sft/:sftId" element={<SftDetailLayout />} />
            <Route
              path="/mint/sft/self-staking/:sftId"
              element={<SftDetailLayoutForSelfStaking />}
            />
            <Route path="/apps" element={<AppsLayout />} />
            <Route path="/transactions" element={<TransactionsLayout />} />
            <Route path="/settings" element={<SettingsLayout />} />
          </Routes>
        </div>
      </ScrollArea>
      <Foot />
    </div>
  )
}
