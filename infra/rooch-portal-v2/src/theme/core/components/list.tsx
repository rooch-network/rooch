import type { Theme, Components } from '@mui/material/styles';

const MuiListItemIcon: Components<Theme>['MuiListItemIcon'] = {
  styleOverrides: {
    root: ({ theme }) => ({ color: 'inherit', minWidth: 'auto', marginRight: theme.spacing(2) }),
  },
};

const MuiListItemAvatar: Components<Theme>['MuiListItemAvatar'] = {
  styleOverrides: { root: ({ theme }) => ({ minWidth: 'auto', marginRight: theme.spacing(2) }) },
};

const MuiListItemText: Components<Theme>['MuiListItemText'] = {
  defaultProps: { primaryTypographyProps: { typography: 'subtitle2' } },

  styleOverrides: { root: { margin: 0 }, multiline: { margin: 0 } },
};

export const list = {
  MuiListItemIcon,
  MuiListItemAvatar,
  MuiListItemText,
};
