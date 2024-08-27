import type { ToggleButtonProps } from '@mui/material/ToggleButton';
import type { Theme, CSSObject, Components } from '@mui/material/styles';

import { toggleButtonClasses } from '@mui/material/ToggleButton';

import { varAlpha } from '../../styles';

const COLORS = ['primary', 'secondary', 'info', 'success', 'warning', 'error'] as const;

type ColorType = (typeof COLORS)[number];

function styleColors(ownerState: ToggleButtonProps, styles: (val: ColorType) => CSSObject) {
  const outputStyle = COLORS.reduce((acc, color) => {
    if (!ownerState.disabled && ownerState.color === color) {
      acc = styles(color);
    }
    return acc;
  }, {});

  return outputStyle;
}

const MuiToggleButton: Components<Theme>['MuiToggleButton'] = {
  styleOverrides: {
    root: ({ theme, ownerState }) => {
      const styled = {
        colors: styleColors(ownerState, (color) => ({
          '&:hover': {
            borderColor: varAlpha(theme.vars.palette[color].mainChannel, 0.48),
            backgroundColor: varAlpha(
              theme.vars.palette[color].mainChannel,
              theme.vars.palette.action.hoverOpacity
            ),
          },
        })),
        selected: {
          [`&.${toggleButtonClasses.selected}`]: {
            borderColor: 'currentColor',
            boxShadow: '0 0 0 0.75px currentColor',
          },
        },
        disabled: {
          ...(ownerState.disabled && {
            [`&.${toggleButtonClasses.selected}`]: {
              color: theme.vars.palette.action.disabled,
              backgroundColor: theme.vars.palette.action.selected,
              borderColor: theme.vars.palette.action.disabledBackground,
            },
          }),
        },
      };

      return { ...styled.colors, ...styled.selected, ...styled.disabled };
    },
  },
};

const MuiToggleButtonGroup: Components<Theme>['MuiToggleButtonGroup'] = {
  styleOverrides: {
    root: ({ theme }) => ({
      gap: 4,
      padding: 4,
      border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.08)}`,
    }),
    grouped: {
      [`&.${toggleButtonClasses.root}`]: { border: 'none', borderRadius: 'inherit' },
      [`&.${toggleButtonClasses.selected}`]: { boxShadow: 'none' },
    },
  },
};

export const toggleButton = { MuiToggleButton, MuiToggleButtonGroup };
