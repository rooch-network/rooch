// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

export const container = style({
  display: 'flex',
  flexDirection: 'row',
  gap: 20,
})

export const infoContainer = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  border: '1px solid #dcdcdc',
  borderRadius: '8px',
  padding: 10,
  maxWidth: 570,
  gap: 4,
})

export const inputContainer = style({
  display: 'flex',
  flexDirection: 'row',
  gap: 12,
  alignItems: 'center',
})

export const separator = style({
  width: '100%',
  height: '1px',
  backgroundColor: '#dcdcdc',
})
