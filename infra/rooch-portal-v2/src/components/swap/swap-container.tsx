import type { ReactNode } from 'react';
import type { SxProps } from '@mui/material';

import { Box, Container } from '@mui/material';

export interface SwapContainerProps {
  width?: number;
  sx?: SxProps;
  children: ReactNode;
}

export default function SwapContainer({ width = 520, sx, children }: SwapContainerProps) {
  return (
    <Container sx={sx}>
      <Box
        sx={{
          margin: 'auto',
          maxWidth: width,
        }}
      >
        {children}
      </Box>
    </Container>
  );
}
