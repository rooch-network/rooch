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

export const input = style({
  backgroundColor: '#1F2937', // 深色背景
  color: '#F9FAFB', // 浅色文字
  border: '1px solid #374151', // 边框颜色
  borderRadius: '6px',
  padding: '8px 12px',
  fontSize: '14px',
  outline: 'none',
  '::placeholder': {
    color: '#9CA3AF', // 占位符颜色
  },
  ':focus': {
    borderColor: '#4B5563', // 聚焦时的边框颜色
  },
})

export const separator = style({
  width: '100%',
  height: '1px',
  backgroundColor: '#dcdcdc',
})
