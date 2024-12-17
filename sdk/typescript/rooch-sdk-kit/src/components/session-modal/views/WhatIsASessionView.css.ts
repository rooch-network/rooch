// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'
import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
})

export const content = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  width: '100%',
  flexGrow: 1,
  gap: 20,
  padding: 20,
})

export const createButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})

export const moreContent = style({
  display: 'flex',
  width: '100%',
  alignItems: 'center',
  flexDirection: 'column',
})

export const moreInfo = style({
  color: themeVars.colors.bodyDanger,
})

export const scopeContent = style({
  display: 'flex',
  flexDirection: 'column',
  flexGrow: 1,
  marginTop: 20,
  padding: 2,
})

export const createSessionStatus = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  height: '100%',
  marginTop: 4,
})

export const retryButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})
