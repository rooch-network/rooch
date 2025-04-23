// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'

// import { themeVars } from '../../../themes/themeContract.js'

export const container = style({
  display: 'flex',
  flexDirection: 'column',
  gap: '1rem',
  padding: '1rem',
  width: '100%',
  alignItems: 'center',
})

export const title = style({
  textAlign: 'center',
  marginBottom: '1rem',
})

export const accountSection = style({
  display: 'flex',
  flexDirection: 'column',
  gap: '1rem',
  marginBottom: '1.5rem',
  padding: '1rem',
  border: '1px solid #e2e8f0',
  borderRadius: '0.375rem',
})

export const accountHeader = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  marginBottom: '0.5rem',
})

export const accountActions = style({
  display: 'flex',
  gap: '0.5rem',
})

export const iconButton = style({
  padding: '0.25rem',
  width: '24px',
  height: '24px',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
})

export const icon = style({
  width: '16px',
  height: '16px',
})

export const accountName = style({
  fontWeight: 600,
  fontSize: '1.1rem',
})

export const addressList = style({
  display: 'flex',
  flexDirection: 'column',
  gap: '0.5rem',
  width: '100%',
  overflowY: 'auto',
  padding: '0 18px',
})

export const addressItem = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '0.5rem',
  border: '1px solid #e2e8f0',
  borderRadius: '0.375rem',
})

export const addressContent = style({
  display: 'flex',
  alignItems: 'center',
  gap: '0.5rem',
})

export const copyFeedback = style({
  fontSize: '0.75rem',
  color: '#10B981',
  opacity: 1,
  transition: 'opacity 0.3s ease-in-out',
})

export const addressText = style({
  fontFamily: 'monospace',
  fontSize: '0.875rem',
  wordBreak: 'break-all',
  width: '100%',
  cursor: 'pointer',
  ':hover': {
    opacity: 0.8,
  },
})

export const actions = style({
  display: 'flex',
  gap: '0.5rem',
  justifyContent: 'center',
})

export const importForm = style({
  display: 'flex',
  flexDirection: 'column',
  gap: '1rem',
  padding: '1rem',
  border: '1px solid #e2e8f0',
  borderRadius: '0.375rem',
})

export const importInput = style({
  padding: '0.5rem',
  border: '1px solid #e2e8f0',
  borderRadius: '0.375rem',
  minHeight: '100px',
  resize: 'vertical',
})

export const importActions = style({
  display: 'flex',
  gap: '0.5rem',
  justifyContent: 'flex-end',
})

export const addressActions = style({
  display: 'flex',
  gap: '0.25rem',
  justifyContent: 'flex-end',
})

export const actionButton = style({
  padding: '0.25rem 0.5rem',
  fontSize: '0.75rem',
  minWidth: 'auto',
})

export const copyNotification = style({
  position: 'fixed',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  backgroundColor: 'rgba(0, 0, 0, 0.8)',
  color: 'white',
  padding: '0.5rem 1rem',
  borderRadius: '0.375rem',
  fontSize: '0.875rem',
  zIndex: 1000,
})
