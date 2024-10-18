














import { Box, Stack, Typography } from '@mui/material';

import { grey } from 'src/theme/core';
import headerIcon from '@/assets/swap/swap-header-icon.svg';

export default function SwapTransactionHeader({ invert = false }: { invert?: boolean }) {
  return (
    <Stack direction="row" alignItems="center" spacing={invert ? '14px' : 1.5}>
      {invert && (
        <Box
          component="img"
          src={headerIcon}
          sx={{
            width: '32px',
            height: '32px',
          }}
        />
      )}
      {!invert && (
        <Box
          component="img"
          src={headerIcon}
          sx={{
            padding: '7px 9px 9px 7px',
            borderRadius: '10px',
            border: `1px solid ${grey[200]}`,
            boxShadow: '0px 1px 2px 0px rgba(16, 24, 40, 0.05)',
          }}
        />
      )}
      <Typography
        sx={{
          fontSize: invert ? '1.8125rem' : '2rem',
          fontWeight: 600,
          lineHeight: invert ? '36px' : '100%',
          color: invert ? '#fff' : grey[900],
        }}
      >
        Swap
      </Typography>
    </Stack>
  );
}
