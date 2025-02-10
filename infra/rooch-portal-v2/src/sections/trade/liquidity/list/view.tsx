'use client';

import { useState } from 'react';

import { Tab, Tabs, Stack, Typography } from '@mui/material';

import { useTabs } from 'src/hooks/use-tabs';

import { DashboardContent } from 'src/layouts/dashboard';

import FarmList from './farm_list';
import AllLiquidityList from './all_liquidity_list';
import OwnerLiquidityList from './owner_liquidity_list';
import CreateLiquidityModal from './create-liquidity-modal';

const TABS = [
  { label: 'All Liquidity', value: 'all_liquidity' },
  { label: 'Your Liquidity', value: 'you_liquidity' },
  { label: 'Farm', value: 'farm' },
];

export default function LiquidityListView() {
  const tabs = useTabs('all_liquidity');

  const [openCreateModal, setOpenCreateModal] = useState(false);

  const handleCloseCreateModal = () => {
    setOpenCreateModal(false);
  };

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 1, md: 1 } }}>
      {TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="row" alignItems="center" justifyContent="space-between">
        <Stack>
          <Typography variant="h4">Pool</Typography>
          {renderTabs}
        </Stack>

        {/* <Button variant="outlined" onClick={handleOpenCreateModal}>
          Create Liquidity
        </Button> */}
      </Stack>

      {tabs.value === 'all_liquidity' && <AllLiquidityList />}
      {tabs.value === 'you_liquidity' && <OwnerLiquidityList />}
      {tabs.value === 'farm' && <FarmList />}

      <CreateLiquidityModal
        open={openCreateModal}
        onClose={handleCloseCreateModal}
        key={openCreateModal ? 'open' : 'closed'}
      />
    </DashboardContent>
  );
}
