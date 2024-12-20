// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  width: '100%',
})

export const title = style({
  marginTop: 16,
})

export const content = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  padding: 20,
})

export const actionButtonContainer = style({
  position: 'absolute',
  bottom: 20,
  right: 20,
})
