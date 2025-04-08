// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { RecipeVariants } from '@vanilla-extract/recipes'
import { recipe } from '@vanilla-extract/recipes'
import { themeVars } from '../../themes/themeContract.js'

export const inputVariants = recipe({
  base: {
    display: 'inline-flex',
    appearance: 'none', // General appearance reset
    alignItems: 'center',
    fontWeight: themeVars.fontWeights.medium,
    border: `1px solid ${themeVars.borderColors.outlineButton}`,
    outline: 'none',
    backgroundColor: themeVars.backgroundColors.modalPrimary,
    color: themeVars.colors.body,
    '::-webkit-inner-spin-button': {
      appearance: 'none', // Remove spinner in Chrome/Safari
      margin: 0,
    },
    '::-webkit-outer-spin-button': {
      appearance: 'none', // Remove spinner in Chrome/Safari
      margin: 0,
    },
    // You can handle focus styles separately using a class
    ':disabled': {
      opacity: 0.5,
      backgroundColor: themeVars.backgroundColors.modalSecondary,
    },
    '::placeholder': {
      color: themeVars.colors.bodyMuted,
    },
    ':focus': {
      borderColor: themeVars.borderColors.outlineButton,
    },
  },
  variants: {
    variant: {
      primary: {
        backgroundColor: themeVars.backgroundColors.modalPrimary,
        color: themeVars.colors.body,
        ':hover': {
          backgroundColor: themeVars.backgroundColors.modalSecondary,
        },
      },
      outline: {
        backgroundColor: themeVars.backgroundColors.modalSecondary,
        borderColor: themeVars.borderColors.outlineButton,
        color: themeVars.colors.body,
        ':hover': {
          backgroundColor: themeVars.backgroundColors.outlineButtonHover,
        },
      },
    },
    size: {
      md: {
        borderRadius: themeVars.radii.medium,
        padding: '8px 12px',
        fontSize: themeVars.fontSizes.small,
      },
      lg: {
        borderRadius: themeVars.radii.large,
        padding: '12px 16px',
        fontSize: themeVars.fontSizes.medium,
      },
    },
  },
  defaultVariants: {
    variant: 'primary',
    size: 'md',
  },
})

export type InputVariants = RecipeVariants<typeof inputVariants>
