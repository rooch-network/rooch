// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  width: '100%',
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

export const connectionStatus = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  height: '100%',
})

export const retryButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})
