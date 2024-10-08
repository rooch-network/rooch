import { Box, Container, SxProps } from '@mui/material';
import { ReactNode } from 'react';

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
