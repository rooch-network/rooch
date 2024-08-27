import type { Theme, Components } from '@mui/material/styles';

const MuiLink: Components<Theme>['MuiLink'] = {
  defaultProps: { underline: 'hover' },

  styleOverrides: {},
};

export const link = { MuiLink };
