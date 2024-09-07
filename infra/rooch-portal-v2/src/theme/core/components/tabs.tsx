import type { Theme, Components } from '@mui/material/styles';

import { tabClasses } from '@mui/material/Tab';

const MuiTabs: Components<Theme>['MuiTabs'] = {
  defaultProps: {
    textColor: 'inherit',
    variant: 'scrollable',
    allowScrollButtonsMobile: true,
  },

  styleOverrides: {
    flexContainer: ({ ownerState, theme }) => ({
      ...(ownerState.variant !== 'fullWidth' && {
        gap: '24px',
        [theme.breakpoints.up('sm')]: {
          gap: '40px',
        },
      }),
    }),
    indicator: { backgroundColor: 'currentColor' },
  },
};

const MuiTab: Components<Theme>['MuiTab'] = {
  defaultProps: { disableRipple: true, iconPosition: 'start' },
  styleOverrides: {
    root: ({ theme }) => ({
      opacity: 1,
      minWidth: 48,
      minHeight: 48,
      padding: theme.spacing(1, 0),
      color: theme.vars.palette.text.secondary,
      fontWeight: theme.typography.fontWeightMedium,
      lineHeight: theme.typography.body2.lineHeight,
      [`&.${tabClasses.selected}`]: {
        color: theme.vars.palette.text.primary,
        fontWeight: theme.typography.fontWeightSemiBold,
      },
    }),
  },
};

export const tabs = { MuiTabs, MuiTab };
