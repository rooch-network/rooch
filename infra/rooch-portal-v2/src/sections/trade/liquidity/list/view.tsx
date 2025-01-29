'use client';

import { Tab, Tabs, Stack, Typography } from '@mui/material';

import { useTabs } from 'src/hooks/use-tabs';

import { DashboardContent } from 'src/layouts/dashboard';

import FarmList from './farm_list';
import AllLiquidityList from './all_liquidity_list';
import OwnerLiquidityList from './owner_liquidity_list';

const TABS = [
  { label: 'All Liquidity', value: 'all_liquidity' },
  { label: 'Your Liquidity', value: 'you_liquidity' },
  { label: 'Farm', value: 'farm' },
];

export default function LiquidityListView() {
  const tabs = useTabs('all_liquidity');

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 1, md: 1 } }}>
      {TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="row" justifyContent="space-between">
        <Typography variant="h4">Pool</Typography>
      </Stack>
      {renderTabs}

      {tabs.value === 'all_liquidity' && <AllLiquidityList />}
      {tabs.value === 'you_liquidity' && <OwnerLiquidityList />}
      {tabs.value === 'farm' && <FarmList />}
    </DashboardContent>
  );
}
