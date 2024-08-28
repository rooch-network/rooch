import type { Theme, Components } from '@mui/material/styles';

const MuiStepConnector: Components<Theme>['MuiStepConnector'] = {
  styleOverrides: { line: ({ theme }) => ({ borderColor: theme.vars.palette.divider }) },
};

export const stepper = { MuiStepConnector };
