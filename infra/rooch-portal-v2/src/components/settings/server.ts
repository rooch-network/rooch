import { cookies } from 'next/headers';

import { STORAGE_KEY, defaultSettings } from './config-settings';

export async function detectSettings() {
  const cookieStore = cookies();

  const settingsStore = cookieStore.get(STORAGE_KEY);

  return settingsStore ? JSON.parse(settingsStore?.value) : defaultSettings;
}
