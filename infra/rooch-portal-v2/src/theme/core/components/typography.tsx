import type { Theme, Components } from '@mui/material/styles';

const MuiTypography: Components<Theme>['MuiTypography'] = {
  styleOverrides: {
    paragraph: ({ theme }) => ({ marginBottom: theme.spacing(2) }),
    gutterBottom: ({ theme }) => ({ marginBottom: theme.spacing(1) }),
  },
};

export const typography = { MuiTypography };
