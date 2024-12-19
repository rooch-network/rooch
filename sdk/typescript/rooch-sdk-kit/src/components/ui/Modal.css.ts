// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { style } from '@vanilla-extract/css'
import { themeVars } from '../../themes/themeContract.js'

export const overlay = style({
  backgroundColor: themeVars.backgroundColors.modalOverlay,
  backdropFilter: themeVars.blurs.modalOverlay,
  position: 'fixed',
  inset: 0,
  zIndex: 999999999,
})

export const content = style({
  backgroundColor: themeVars.backgroundColors.modalPrimary,
  borderRadius: themeVars.radii.xlarge,
  color: themeVars.colors.body,
  position: 'fixed',
  bottom: 16,
  left: 16,
  right: 16,
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'space-between',
  overflow: 'hidden',
  minHeight: '50vh',
  maxHeight: '85vh',
  maxWidth: 700,
  '@media': {
    'screen and (min-width: 768px)': {
      flexDirection: 'row',
      width: '100%',
      top: '50%',
      left: '50%',
      transform: 'translate(-50%, -50%)',
    },
  },
})

export const closeButtonContainer = style({
  position: 'absolute',
  top: 16,
  right: 16,
})
