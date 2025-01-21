// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'
import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  width: '100%',
  marginTop: 16,
})

export const content = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  flexGrow: 1,
  gap: 20,
  padding: 40,
})

export const createButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})

export const progressBar = style({
  position: 'absolute',
  bottom: 0,
  left: 10,
  height: '1px',
  backgroundColor: themeVars.colors.primaryButton,
  transition: 'width 0.2s ease',
  borderRadius: '8px',
})
