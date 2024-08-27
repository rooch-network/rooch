'use client';

import { getInitColorSchemeScript as _getInitColorSchemeScript } from '@mui/material/styles';

export const schemeConfig = {
  modeStorageKey: 'theme-mode',
  defaultMode: 'light' as const,
};

export const getInitColorSchemeScript = _getInitColorSchemeScript(schemeConfig);
