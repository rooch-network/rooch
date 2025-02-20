'use client';

import { Box } from '@mui/material';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { useRouter } from 'src/routes/hooks';

import MarketplaceItemCard from 'src/components/market/marketplace-item-card';

const MARKETPLACE_LIST = [
  {
    tick: 'grow',
  },
  {
    tick: 'gold',
  },
];

export default function MarketplaceHomeView() {
  const router = useRouter();

  return (
    <Container maxWidth="xl">
      <Typography variant="h4">Marketplace List</Typography>

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
        {MARKETPLACE_LIST.map((item) => (
          <MarketplaceItemCard
            key={item.tick}
            tick={item.tick}
            onClick={() => {
              router.push(`/trade/market/${item.tick}`);
            }}
          />
        ))}
      </Box>
    </Container>
  );
}
