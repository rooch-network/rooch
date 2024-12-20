// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
})

export const sessionItem = style({
  display: 'flex',
  alignItems: 'center',
  flexGrow: 1,
  padding: 8,
  gap: 8,
  borderRadius: themeVars.radii.large,
  ':hover': {
    backgroundColor: themeVars.backgroundColors.walletItemHover,
  },
  ':focus': {
    outline: 'none',
  },
})

export const selectedSessionItem = style({
  backgroundColor: themeVars.backgroundColors.walletItemSelected,
  boxShadow: '0px 2px 6px rgba(0, 0, 0, 0.05)',
})
