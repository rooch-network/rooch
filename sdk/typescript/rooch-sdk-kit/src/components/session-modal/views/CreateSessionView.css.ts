// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

import { themeVars } from '../../../themes/themeContract.js'
export const container = style({
  display: 'flex',
  width: '100%',
  alignItems: 'center',
  flexDirection: 'column',
})

export const title = style({
  marginTop: 12,
})

export const content = style({
  display: 'flex',
  marginTop: 20,
  flexDirection: 'column',
})

export const sessionItemContent = style({
  display: 'flex',
  flexDirection: 'row',
  flexGrow: 1,
  marginTop: 2,
  padding: 2,
})

export const scopeContent = style({
  display: 'flex',
  flexDirection: 'column',
  flexGrow: 1,
  marginTop: 20,
  padding: 2,
})

export const moreContent = style({
  display: 'flex',
  width: '100%',
  alignItems: 'center',
  flexDirection: 'column',
  marginTop: 10,
})

export const moreInfo = style({
  color: themeVars.colors.bodyDanger,
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
