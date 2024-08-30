import type { Theme, Components } from '@mui/material/styles';

import { inputLabelClasses } from '@mui/material/InputLabel';

const MuiFormLabel: Components<Theme>['MuiFormLabel'] = {
  styleOverrides: {
    root: ({ theme }) => ({
      ...theme.typography.body2,
      color: theme.vars.palette.text.disabled,
      [`&.${inputLabelClasses.shrink}`]: {
        ...theme.typography.body1,
        fontWeight: 600,
        color: theme.vars.palette.text.secondary,
        [`&.${inputLabelClasses.focused}`]: { color: theme.vars.palette.text.primary },
        [`&.${inputLabelClasses.error}`]: { color: theme.vars.palette.error.main },
        [`&.${inputLabelClasses.disabled}`]: { color: theme.vars.palette.text.disabled },
        [`&.${inputLabelClasses.filled}`]: { transform: 'translate(12px, 6px) scale(0.75)' },
      },
    }),
  },
};

const MuiFormHelperText: Components<Theme>['MuiFormHelperText'] = {
  defaultProps: { component: 'div' },

  styleOverrides: { root: ({ theme }) => ({ marginTop: theme.spacing(1) }) },
};

const MuiFormControlLabel: Components<Theme>['MuiFormControlLabel'] = {
  styleOverrides: { label: ({ theme }) => ({ ...theme.typography.body2 }) },
};

export const form = { MuiFormLabel, MuiFormHelperText, MuiFormControlLabel };
