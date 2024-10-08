import type { TypographyProps } from '@mui/material';

import { Typography } from '@mui/material';

export default function Label({ sx, children, onClick }: TypographyProps) {
  return (
    <Typography
      className="text-gray-900"
      sx={{
        fontSize: '0.875rem',
        fontWeight: 400,
        lineHeight: '140%',
        ...sx,
      }}
      onClick={onClick}
    >
      {children}
    </Typography>
  );
}
