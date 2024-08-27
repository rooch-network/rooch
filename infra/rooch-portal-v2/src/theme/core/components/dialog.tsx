import type { Theme, Components } from '@mui/material/styles';

const MuiDialog: Components<Theme>['MuiDialog'] = {
  styleOverrides: {
    paper: ({ ownerState, theme }) => ({
      boxShadow: theme.customShadows.dialog,
      borderRadius: theme.shape.borderRadius * 2,
      ...(!ownerState.fullScreen && { margin: theme.spacing(2) }),
    }),
    paperFullScreen: { borderRadius: 0 },
  },
};

const MuiDialogTitle: Components<Theme>['MuiDialogTitle'] = {
  styleOverrides: { root: ({ theme }) => ({ padding: theme.spacing(3) }) },
};

const MuiDialogContent: Components<Theme>['MuiDialogContent'] = {
  styleOverrides: {
    root: ({ theme }) => ({ padding: theme.spacing(0, 3) }),
    dividers: ({ theme }) => ({
      borderTop: 0,
      borderBottomStyle: 'dashed',
      paddingBottom: theme.spacing(3),
    }),
  },
};

const MuiDialogActions: Components<Theme>['MuiDialogActions'] = {
  defaultProps: { disableSpacing: true },

  styleOverrides: {
    root: ({ theme }) => ({
      padding: theme.spacing(3),
      '& > :not(:first-of-type)': { marginLeft: theme.spacing(1.5) },
    }),
  },
};

export const dialog = {
  MuiDialog,
  MuiDialogTitle,
  MuiDialogContent,
  MuiDialogActions,
};
