import { defaultFont } from 'src/theme/core/typography';

import type { SettingsState } from './types';

export const STORAGE_KEY = 'app-settings';

export const defaultSettings: SettingsState = {
  colorScheme: 'light',
  direction: 'ltr',
  contrast: 'default',
  navLayout: 'vertical',
  primaryColor: 'rooch',
  navColor: 'integrate',
  compactLayout: true,
  fontFamily: defaultFont,
} as const;
