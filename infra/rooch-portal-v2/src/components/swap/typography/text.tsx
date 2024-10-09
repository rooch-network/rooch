import type { TypographyProps } from '@mui/material';

import { Typography } from '@mui/material';

export default function Text({ sx, children, className }: TypographyProps) {
  return (
    <Typography
      className={`text-gray-900 ${className}`}
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
