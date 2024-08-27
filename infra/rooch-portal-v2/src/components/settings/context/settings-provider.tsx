'use client';

import { useMemo, useState, useCallback, createContext } from 'react';

import { useCookies } from 'src/hooks/use-cookies';
import { useLocalStorage } from 'src/hooks/use-local-storage';

import { STORAGE_KEY, defaultSettings } from '../config-settings';

import type { SettingsState, SettingsContextValue, SettingsProviderProps } from '../types';

export const SettingsContext = createContext<SettingsContextValue | undefined>(undefined);

export const SettingsConsumer = SettingsContext.Consumer;

export function SettingsProvider({
  children,
  settings,
  caches = 'localStorage',
}: SettingsProviderProps) {
  const cookies = useCookies<SettingsState>(STORAGE_KEY, settings, defaultSettings);

  const localStorage = useLocalStorage<SettingsState>(STORAGE_KEY, settings);

  const values = caches === 'cookie' ? cookies : localStorage;

  const [openDrawer, setOpenDrawer] = useState(false);

  const onToggleDrawer = useCallback(() => {
    setOpenDrawer((prev) => !prev);
  }, []);

  const onCloseDrawer = useCallback(() => {
    setOpenDrawer(false);
  }, []);

  const memoizedValue = useMemo(
    () => ({
      ...values.state,
      canReset: values.canReset,
      onReset: values.resetState,
      onUpdate: values.setState,
      onUpdateField: values.setField,
      openDrawer,
      onCloseDrawer,
      onToggleDrawer,
    }),
    [
      values.canReset,
      values.resetState,
      values.setField,
      values.setState,
      values.state,
      openDrawer,
      onCloseDrawer,
      onToggleDrawer,
    ]
  );

  return <SettingsContext.Provider value={memoizedValue}>{children}</SettingsContext.Provider>;
}
