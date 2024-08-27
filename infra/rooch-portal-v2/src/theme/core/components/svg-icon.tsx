import type { Theme, Components } from '@mui/material/styles';

const MuiSvgIcon: Components<Theme>['MuiSvgIcon'] = {
  styleOverrides: { fontSizeLarge: { width: 32, height: 32, fontSize: 'inherit' } },
};

export const svgIcon = { MuiSvgIcon };
