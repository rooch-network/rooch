// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style, keyframes } from '@vanilla-extract/css'

import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
})

export const walletItem = style({
  display: 'flex',
  alignItems: 'center',
  flexGrow: 1,
  padding: 12,
  gap: 12,
  borderRadius: themeVars.radii.large,
  transition: 'all 0.2s ease',
  cursor: 'pointer',
  ':hover': {
    backgroundColor: themeVars.backgroundColors.walletItemHover,
    transform: 'translateY(-1px)',
  },
})

export const selectedWalletItem = style({
  backgroundColor: themeVars.backgroundColors.walletItemSelected,
  boxShadow: '0px 2px 6px rgba(0, 0, 0, 0.05)',
})

export const walletIcon = style({
  width: 28,
  height: 28,
  flexShrink: 0,
  objectFit: 'cover',
  borderRadius: themeVars.radii.small,
})

export const walletStatus = style({
  marginLeft: 'auto',
  fontSize: '0.875rem',
  color: themeVars.colors.bodyMuted,
  display: 'flex',
  alignItems: 'center',
  gap: 4,
})

export const installedStatus = style({
  color: themeVars.colors.body,
})

export const notInstalledStatus = style({
  color: themeVars.colors.bodyWarning,
})

export const detectingStatus = style({
  color: themeVars.colors.bodyDanger,
})

const spinAnimation = keyframes({
  from: { transform: 'rotate(0deg)' },
  to: { transform: 'rotate(360deg)' },
})

export const loadingSpinner = style({
  width: 12,
  height: 12,
  border: `2px solid ${themeVars.colors.body}`,
  borderTopColor: 'transparent',
  borderRadius: '50%',
  animation: `${spinAnimation} 1s linear infinite`,
})
