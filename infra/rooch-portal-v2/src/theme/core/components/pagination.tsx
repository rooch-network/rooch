import type { Theme, Components, ComponentsVariants } from '@mui/material/styles';

import { paginationItemClasses } from '@mui/material/PaginationItem';

import { varAlpha, stylesMode } from '../../styles';

declare module '@mui/material/Pagination' {
  interface PaginationPropsVariantOverrides {
    soft: true;
  }

  interface PaginationPropsColorOverrides {
    info: true;
    success: true;
    warning: true;
    error: true;
  }
}

const COLORS = ['primary', 'secondary', 'info', 'success', 'warning', 'error'] as const;

const softVariant: Record<string, ComponentsVariants<Theme>['MuiPagination']> = {
  colors: COLORS.map((color) => ({
    props: ({ ownerState }) =>
      !ownerState.disabled && ownerState.variant === 'soft' && ownerState.color === color,
    style: ({ theme }) => ({
      [`& .${paginationItemClasses.root}`]: {
        [`&.${paginationItemClasses.selected}`]: {
          fontWeight: theme.typography.fontWeightSemiBold,
          color: theme.vars.palette[color].dark,
          backgroundColor: varAlpha(theme.vars.palette[color].mainChannel, 0.08),
          '&:hover': { backgroundColor: varAlpha(theme.vars.palette[color].mainChannel, 0.16) },
          [stylesMode.dark]: { color: theme.vars.palette[color].light },
        },
      },
    }),
  })),
  standardColor: [
    {
      props: ({ ownerState }) => ownerState.variant === 'soft' && ownerState.color === 'standard',
      style: ({ theme }) => ({
        [`& .${paginationItemClasses.root}`]: {
          [`&.${paginationItemClasses.selected}`]: {
            fontWeight: theme.typography.fontWeightSemiBold,
            backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
            '&:hover': { backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16) },
          },
        },
      }),
    },
  ],
};

const MuiPagination: Components<Theme>['MuiPagination'] = {
  variants: [
    /**
     * @variant soft
     */
    ...[...softVariant.standardColor!, ...softVariant.colors!],
  ],

  styleOverrides: {
    /**
     * @variant text
     */
    text: ({ ownerState, theme }) => ({
      [`& .${paginationItemClasses.root}`]: {
        [`&.${paginationItemClasses.selected}`]: {
          fontWeight: theme.typography.fontWeightSemiBold,
          ...(ownerState.color === 'standard' && {
            color: theme.vars.palette.common.white,
            backgroundColor: theme.vars.palette.text.primary,
            '&:hover': { backgroundColor: theme.vars.palette.grey[700] },
            [stylesMode.dark]: {
              color: theme.vars.palette.grey[800],
              '&:hover': { backgroundColor: theme.vars.palette.grey[100] },
            },
          }),
        },
      },
    }),
    /**
     * @variant outlined
     */
    outlined: ({ ownerState, theme }) => ({
      [`& .${paginationItemClasses.root}`]: {
        borderColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.24),
        [`&.${paginationItemClasses.selected}`]: {
          borderColor: 'currentColor',
          fontWeight: theme.typography.fontWeightSemiBold,
          ...(ownerState.color === 'standard' && {
            backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
          }),
        },
      },
    }),
  },
};

export const pagination = { MuiPagination };
