import type { Theme, Components } from '@mui/material/styles';

import { badgeClasses } from '@mui/material/Badge';

declare module '@mui/material/Badge' {
  interface BadgePropsVariantOverrides {
    alway: true;
    busy: true;
    online: true;
    offline: true;
    invisible: true;
  }
}

const baseStyles = (theme: Theme) => ({
  width: 10,
  zIndex: 9,
  padding: 0,
  height: 10,
  minWidth: 'auto',
  '&::before, &::after': {
    content: "''",
    borderRadius: 1,
    backgroundColor: theme.vars.palette.common.white,
  },
  [`&.${badgeClasses.invisible}`]: { transform: 'unset' },
});

const MuiBadge: Components<Theme>['MuiBadge'] = {
  variants: [
    /**
     * @variant online
     */
    {
      props: ({ ownerState }) => ownerState.variant === 'online',
      style: ({ theme }) => ({
        [`& .${badgeClasses.badge}`]: {
          ...baseStyles(theme),
          backgroundColor: theme.vars.palette.success.main,
        },
      }),
    },
    /**
     * @variant alway
     */
    {
      props: ({ ownerState }) => ownerState.variant === 'alway',
      style: ({ theme }) => ({
        [`& .${badgeClasses.badge}`]: {
          ...baseStyles(theme),
          backgroundColor: theme.vars.palette.warning.main,
          '&::before': { width: 2, height: 4, transform: 'translateX(1px) translateY(-1px)' },
          '&::after': { width: 2, height: 4, transform: 'translateY(1px) rotate(125deg)' },
        },
      }),
    },
    /**
     * @variant busy
     */
    {
      props: ({ ownerState }) => ownerState.variant === 'busy',
      style: ({ theme }) => ({
        [`& .${badgeClasses.badge}`]: {
          ...baseStyles(theme),
          backgroundColor: theme.vars.palette.error.main,
          '&::before': { width: 6, height: 2 },
        },
      }),
    },
    /**
     * @variant offline
     */
    {
      props: ({ ownerState }) => ownerState.variant === 'offline',
      style: ({ theme }) => ({
        [`& .${badgeClasses.badge}`]: {
          ...baseStyles(theme),
          backgroundColor: theme.vars.palette.text.disabled,
          '&::before': { width: 6, height: 6, borderRadius: '50%' },
        },
      }),
    },
    /**
     * @variant invisible
     */
    {
      props: ({ ownerState }) => ownerState.variant === 'invisible',
      style: { [`& .${badgeClasses.badge}`]: { display: 'none' } },
    },
  ],

  styleOverrides: { dot: { borderRadius: '50%' } },
};

export const badge = { MuiBadge };
