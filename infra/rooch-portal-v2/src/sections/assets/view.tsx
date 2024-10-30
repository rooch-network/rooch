'use client';

import { Tab, Tabs, Stack } from '@mui/material';
import Typography from '@mui/material/Typography';

import useAddressChanged from 'src/routes/hooks/useAddressChanged';

import { useTabs } from 'src/hooks/use-tabs';

import { DashboardContent } from 'src/layouts/dashboard';

import NFTList from './components/nft-list-card';
import UTXOList from './components/utxo-list-card';
import OrdinalList from './components/ordinal-list-card';
import AssetsTableCard from './components/assets-table-card';

export type IDateValue = string | number | null;

const ASSETS_VIEW_TABS = [
  { label: 'Coin', value: 'coin' },
  { label: 'NFT', value: 'nft' },
  { label: 'Bitcoin Assets', value: 'bitcoin' },
];

export function AssetsView({ address }: { address: string }) {
  const tabs = useTabs('coin');

  useAddressChanged({ address, path: 'assets' });

  const renderTabs = (
    <Tabs value={tabs.value} onChange={tabs.onChange} sx={{ mb: { xs: 1, md: 1 } }}>
      {ASSETS_VIEW_TABS.map((tab) => (
        <Tab key={tab.value} value={tab.value} label={tab.label} />
      ))}
    </Tabs>
  );

  return (
    <DashboardContent maxWidth="xl">
      <Stack flexDirection="row" justifyContent="space-between">
        <Typography variant="h4">
          Assets
          <span className="text-gray-400 text-sm ml-2">({address})</span>
        </Typography>
      </Stack>

      {renderTabs}

      {tabs.value === 'coin' && <AssetsTableCard address={address} />}

      {tabs.value === 'nft' && <NFTList address={address} />}

      {tabs.value === 'bitcoin' && (
        <Stack spacing={2}>
          <UTXOList address={address} />
          <OrdinalList address={address} />
        </Stack>
      )}
    </DashboardContent>
  );
}
