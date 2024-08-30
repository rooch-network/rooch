import type { Theme, Components } from '@mui/material/styles';

import { listClasses } from '@mui/material/List';

import { paper } from '../../styles';

const MuiPopover: Components<Theme>['MuiPopover'] = {
  styleOverrides: {
    paper: ({ theme }) => ({
      ...paper({ theme, dropdown: true }),
      [`& .${listClasses.root}`]: { paddingTop: 0, paddingBottom: 0 },
    }),
  },
};

export const popover = { MuiPopover };
