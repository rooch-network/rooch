// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ThemeVars } from './themeContract.js'

export const darkTheme: ThemeVars = {
  blurs: {
    modalOverlay: 'blur(2px)',
  },
  backgroundColors: {
    primaryButton: '#1F2937',
    primaryButtonHover: '#374151',
    outlineButtonHover: '#4B5563',
    modalOverlay: 'rgba(0, 0, 0, 0.7)',
    modalPrimary: '#111827',
    modalSecondary: '#1F2937',
    iconButton: 'transparent',
    iconButtonHover: '#374151',
    dropdownMenu: '#1F2937',
    dropdownMenuSeparator: '#374151',
    walletItemSelected: '#374151',
    walletItemHover: '#4B556322',
  },
  borderColors: {
    outlineButton: '#4B5563',
  },
  colors: {
    primaryButton: '#F9FAFB',
    outlineButton: '#F9FAFB',
    iconButton: '#FFFFFF',
    body: '#D1D5DB',
    bodyMuted: '#9CA3AF',
    bodyDanger: '#F87171',
    bodyWarning: '#FBBF24',
  },
  radii: {
    small: '6px',
    medium: '8px',
    large: '12px',
    xlarge: '16px',
  },
  shadows: {
    primaryButton: '0px 4px 12px rgba(0, 0, 0, 0.3)',
    walletItemSelected: '0px 2px 6px rgba(0, 0, 0, 0.2)',
  },
  fontWeights: {
    normal: '400',
    medium: '500',
    bold: '600',
  },
  fontSizes: {
    small: '14px',
    medium: '16px',
    large: '18px',
    xlarge: '20px',
  },
  typography: {
    fontFamily:
      'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"',
    fontStyle: 'normal',
    lineHeight: '1.3',
    letterSpacing: '1',
  },
}
