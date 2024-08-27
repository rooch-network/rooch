'use client';

import { useMemo } from 'react';

import { Tab, Tabs, Stack } from '@mui/material';
import Typography from '@mui/material/Typography';

import { useTabs } from 'src/hooks/use-tabs';

import { DashboardContent } from 'src/layouts/dashboard';

import MintTableCard from './components/mint-table-card';

export type IDateValue = string | number | null;

const ASSETS_VIEW_TABS = [
  { label: 'Demo', value: 'demo' },
  { label: 'Coming Soon', value: 'comingSoon' },
];

export default function MintView() {
  const tabs = useTabs('demo');

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 1, md: 1 } }}>
      {ASSETS_VIEW_TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  const memoizedMintTableCard = useMemo(
    () => ({
      demo: <MintTableCard key="demo" />,
      comingSoon: <MintTableCard isStaticData key="comingSoon" />,
    }),
    []
  );

  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="row" justifyContent="space-between">
        <Typography variant="h4">Mint</Typography>
      </Stack>

      {renderTabs}

      {memoizedMintTableCard[tabs.value as keyof typeof memoizedMintTableCard]}
    </DashboardContent>
  );
}
