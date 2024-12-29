import { useCurrentAddress, useSignAndExecuteTransaction } from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import { grey, yellow } from '@mui/material/colors';
import { Card, Chip, Stack, Typography, CardActions } from '@mui/material';

import { secondary } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

export type MarketplaceItemCardProps = {
  tick: string;
  onClick: () => void;
};

export default function MarketplaceItemCard({ tick, onClick }: MarketplaceItemCardProps) {
  const account = useCurrentAddress();
  const { mutate: signAndExecuteTransaction, isPending } = useSignAndExecuteTransaction();

  const isVerified = true;

  return (
    <Card
      key={tick}
      sx={{
        '&:hover .add-cart-btn': {
          opacity: 1,
        },
        p: 1,
        cursor: 'pointer',
      }}
      onClick={() => {
        onClick();
      }}
    >
      <Stack
        justifyContent="center"
        alignItems="center"
        spacing={1}
        sx={{
          p: 1,
          borderRadius: '4px',
        }}
      >
        <Stack
          sx={{
            width: '100%',
          }}
          direction="row"
          alignItems="center"
          justifyContent="space-between"
        >
          <Chip
            size="small"
            label={
              <Stack
                direction="row"
                alignItems="center"
                sx={{
                  fontSize: {
                    xs: '0.75rem',
                    sm: '0.8125rem',
                  },
                }}
              >
                {tick.toUpperCase()}
                {isVerified && (
                  <Iconify
                    icon="solar:verified-check-bold"
                    color={yellow.A200}
                    width={16}
                    sx={{
                      ml: 0.5,
                    }}
                  />
                )}
              </Stack>
            }
            variant="filled"
            color="secondary"
          />
        </Stack>
        <Typography
          sx={{
            fontSize: '2rem',
            fontWeight: 600,
          }}
        >
          {tick.toUpperCase()}
        </Typography>

        <Typography
          sx={{
            fontWeight: '400',
            fontSize: '0.875rem',
            color: grey[600],
            display: 'flex',
            alignItems: 'center',
          }}
        >
          <Typography
            sx={{
              mr: 1,
              fontSize: '1rem',
              color: secondary.light,
            }}
          >
            RGAS/{tick.toUpperCase()}
          </Typography>
        </Typography>
      </Stack>
      <CardActions>
        <Stack
          direction="row"
          sx={{
            width: '100%',
          }}
          justifyContent="space-around"
          spacing={2}
        >
          <LoadingButton
            loading={isPending}
            variant="outlined"
            color="primary"
            fullWidth
            size="small"
            onClick={() => {
              onClick();
            }}
          >
            Trade
          </LoadingButton>
        </Stack>
      </CardActions>
    </Card>
  );
}
