// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

export const card = style({
  backgroundColor: '#ffffff',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0, 0, 0, 0.08)',
  padding: '16px',
  maxWidth: '280px',
  margin: '16px auto',
  border: '1px solid #dcdcdc',
  display: 'flex',
  flexDirection: 'column',
  gap: '8px',
})

export const cardHeader = style({
  display: 'flex',
  justifyContent: 'space-between', // Space between header sections
  alignItems: 'center', // Center vertically
  fontSize: '1.25rem',
  fontWeight: 'bold',
})

export const cardHeaderLeft = style({
  // Additional styles (if needed)
})

export const cardHeaderRight = style({
  // Additional styles (if needed)
})

export const cardBody = style({
  fontSize: '1rem',
  color: '#333',
})

export const cardFooter = style({
  marginTop: '12px',
  textAlign: 'right',
  fontSize: '0.875rem',
  color: '#007BFF',
})
