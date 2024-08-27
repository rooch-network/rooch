import type { Theme, Components } from '@mui/material/styles';

import { varAlpha } from '../../styles';

const MuiSkeleton: Components<Theme>['MuiSkeleton'] = {
  defaultProps: { animation: 'wave', variant: 'rounded' },
  styleOverrides: {
    root: ({ theme }) => ({
      backgroundColor: varAlpha(theme.vars.palette.grey['400Channel'], 0.12),
    }),
    rounded: ({ theme }) => ({ borderRadius: theme.shape.borderRadius * 2 }),
  },
};

export const skeleton = { MuiSkeleton };
