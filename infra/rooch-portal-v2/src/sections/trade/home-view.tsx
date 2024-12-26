'use client';

import { Box } from '@mui/material';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { useRouter } from 'src/routes/hooks';

import { useSettingsContext } from 'src/components/settings';
import MarketplaceItemCard from 'src/components/market/markerplace-item-card';

export interface Tick {
  current_epoch: string;
  current_supply: string;
  epoch_count: string;
  epoch_records: {
    type: string;
    fields: {
      id: {
        id: string;
      };
      size: string;
    };
  };
  id: {
    id: string;
  };
  mint_fee: string;
  remain: string;
  start_time_ms: string;
  tick: string;
  total_supply: string;
  total_transactions: string;
  version: string;
}

// ----------------------------------------------------------------------
const TABLE_HEAD = [
  { id: 'name', label: 'Name' },
  { id: 'todayVolume	', label: 'Today Volume (SUI)', align: 'center' },
  { id: 'totalVolume', label: 'Total Volume (SUI)', align: 'right' },
  { id: 'totalSupply', label: 'Total Supply', align: 'right' },
  { id: 'action', label: 'Action', align: 'center' },
];

export default function MarketplaceHomeView() {
  const settings = useSettingsContext();
  const router = useRouter();
  // const { tickList: ticks, isFetching } = useMRCTicks();
  // const { tickTradeInfos, isLoadingTickTradeInfos } = useBatchMarketTradeData([
  //   'grow',
  //   'gold',
  // ]);

  return (
    <Container maxWidth="xl">
      <Typography variant="h4"> Marketplace List</Typography>

      <Box
        gap={3}
        display="grid"
        gridTemplateColumns={{
          xs: 'repeat(2, 1fr)',
          sm: 'repeat(3, 1fr)',
          md: 'repeat(3, 1fr)',
          lg: 'repeat(3, 1fr)',
        }}
        sx={{
          mt: 2,
        }}
      >
        <MarketplaceItemCard
          tick="grow"
          onClick={() => {
            router.push(`/trade/grow`);
          }}
        />
        <MarketplaceItemCard
          tick="gold"
          onClick={() => {
            router.push(`/trade/gold`);
          }}
        />
      </Box>
    </Container>
  );
}
