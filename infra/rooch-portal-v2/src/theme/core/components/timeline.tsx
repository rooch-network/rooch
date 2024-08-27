import type { Theme, Components } from '@mui/material/styles';

const MuiTimelineDot: Components<Theme>['MuiTimelineDot'] = {
  styleOverrides: { root: { boxShadow: 'none' } },
};

const MuiTimelineConnector: Components<Theme>['MuiTimelineConnector'] = {
  styleOverrides: { root: ({ theme }) => ({ backgroundColor: theme.vars.palette.divider }) },
};

export const timeline = { MuiTimelineDot, MuiTimelineConnector };
