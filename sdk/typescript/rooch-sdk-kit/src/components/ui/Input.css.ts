// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { RecipeVariants } from '@vanilla-extract/recipes'
import { recipe } from '@vanilla-extract/recipes'

export const inputVariants = recipe({
  base: {
    display: 'inline-flex',
    appearance: 'none', // General appearance reset
    alignItems: 'center',
    fontWeight: '500', // Medium weight
    border: '1px solid #ccc', // Default input border color
    outline: 'none',
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
      backgroundColor: '#f5f5f5', // Disabled background color
    },
  },
  variants: {
    variant: {
      primary: {
        backgroundColor: '#ffffff', // Primary background color
        color: '#000000', // Primary text color
        ':hover': {
          backgroundColor: '#e6f7ff', // Hover background color
        },
      },
      outline: {
        backgroundColor: '#f8f9fa', // Outline background color
        borderColor: '#007BFF', // Outline border color
        color: '#007BFF', // Outline text color
        ':hover': {
          backgroundColor: '#e2e6ea', // Outline hover background color
        },
      },
    },
    size: {
      md: {
        borderRadius: '4px',
        padding: '8px 12px',
      },
      lg: {
        borderRadius: '8px',
        padding: '12px 16px',
      },
    },
  },
  defaultVariants: {
    variant: 'primary',
    size: 'md',
  },
})

export type InputVariants = RecipeVariants<typeof inputVariants>
