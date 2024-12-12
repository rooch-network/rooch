// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
})

export const walletIcon = style({
  objectFit: 'cover',
  width: 72,
  height: 72,
  borderRadius: themeVars.radii.large,
})

export const title = style({
  marginTop: 12,
})

export const content = style({
  display: 'flex',
  flexDirection: 'column',
})

export const sessionItemContent = style({
  display: 'flex',
  flexDirection: 'row',
  flexGrow: 1,
  top: 20,
  padding: 2,
})

export const ScopeContent = style({
  display: 'flex',
  flexDirection: 'column',
  flexGrow: 1,
  marginTop: 20,
  padding: 2,
})

export const removeButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})
