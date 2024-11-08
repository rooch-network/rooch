import type { PaperProps } from '@mui/material';

import { Paper, Stack, Skeleton } from '@mui/material';

export function ProductItemSkeleton({ sx, ...other }: PaperProps) {
  return (
    <Paper
      variant="outlined"
      sx={{
        borderRadius: 2,
        ...sx,
      }}
      {...other}
    >
      <Stack spacing={2} sx={{ p: 3, pt: 2, width: '100%' }} alignItems="center">
        <Stack
          direction="row"
          justifyContent="space-between"
          sx={{
            width: '100%',
          }}
        >
          <Skeleton sx={{ width: 54.15, height: 19.15 }} />
          <Skeleton sx={{ width: 54.15, height: 19.15 }} />
        </Stack>
        <Skeleton variant="text" sx={{ width: 128, height: 36 }} />
        <Stack
          direction="row"
          sx={{
            width: '100%',
            mt: 3,
          }}
          justifyContent="space-around"
        >
          <Skeleton sx={{ width: 200, height: 30 }} />
        </Stack>
      </Stack>
    </Paper>
  );
}
