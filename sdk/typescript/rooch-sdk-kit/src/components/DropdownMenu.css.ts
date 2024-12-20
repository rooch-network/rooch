// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

import { themeVars } from '../themes/themeContract.js'

export const connectedAddress = style({
  gap: 8,
})

export const menuContainer = style({
  zIndex: 999999999,
})

export const menuContent = style({
  display: 'flex',
  flexDirection: 'column',
  width: 180,
  maxHeight: 200,
  marginTop: 4,
  padding: 8,
  gap: 8,
  borderRadius: themeVars.radii.large,
  backgroundColor: themeVars.backgroundColors.dropdownMenu,
})

export const menuItem = style({
  padding: 8,
  userSelect: 'none',
  outline: 'none',
  display: 'flex',
  alignItems: 'center',
  textAlign: 'center',
  borderRadius: themeVars.radii.large,
  selectors: {
    '&[data-highlighted]': {
      backgroundColor: themeVars.backgroundColors.primaryButton,
    },
  },
})

export const switchMenuItem = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
})

export const separator = style({
  height: 1,
  flexShrink: 0,
  backgroundColor: themeVars.backgroundColors.dropdownMenuSeparator,
})

export const progressBar = style({
  position: 'absolute',
  bottom: 0,
  left: 10,
  height: '0.5px',
  backgroundColor: themeVars.colors.primaryButton,
  transition: 'width 0.2s ease',
  borderRadius: '8px',
})
export const addressContainer = style({
  display: 'flex',
  flexDirection: 'column',
})
export const rgasBalance = style({
  fontSize: '0.5rem',
  color: '#666',
})
