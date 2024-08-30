import type { Theme, Components } from '@mui/material/styles';

const MuiTreeItem: Components<Theme>['MuiTreeItem'] = {
  styleOverrides: {
    label: ({ theme }) => ({ ...theme.typography.body2 }),
    iconContainer: { width: 'auto' },
  },
};

export const treeView = { MuiTreeItem };
