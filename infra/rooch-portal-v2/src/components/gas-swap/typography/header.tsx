import type { TypographyProps } from '@mui/material';

import { Typography } from '@mui/material';

export default function Header({ sx, children }: TypographyProps) {
  return (
    <Typography
      className="text-gray-900"
      sx={{
        fontSize: '1rem',
        fontWeight: 600,
        lineHeight: '24px',
        ...sx,
      }}
    >
      {children}
    </Typography>
  );
}
