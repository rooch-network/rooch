import type { TypographyProps } from '@mui/material';

import { Typography } from '@mui/material';

export default function Text({ sx, children }: TypographyProps) {
  return (
    <Typography
      className="text-gray-900"
      sx={{
        fontSize: '0.875rem',
        fontWeight: 500,
        lineHeight: '24px',
        ...sx,
      }}
    >
      {children}
    </Typography>
  );
}
