import 'src/global.css';
import '@fontsource-variable/raleway/wght.css';
import '@fontsource-variable/plus-jakarta-sans/wght.css';

import type { Viewport } from 'next';

import { headers } from 'next/headers';
import '@fontsource-variable/red-hat-mono';

import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';

import { primary } from 'src/theme/core/palette';
import { DashboardLayout } from 'src/layouts/dashboard';
import { ThemeProvider } from 'src/theme/theme-provider';
import { schemeConfig } from 'src/theme/color-scheme-script';

import { Snackbar } from 'src/components/snackbar';
import { ProgressBar } from 'src/components/progress-bar';
import { MotionLazy } from 'src/components/animate/motion-lazy';
import { SettingsDrawer, defaultSettings, SettingsProvider } from 'src/components/settings';

import RoochDappProvider from './rooch-dapp-provider';

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: primary.main,
};

type Props = {
  children: React.ReactNode;
};

export default async function RootLayout({ children }: Props) {
  // const settings = CONFIG.isStaticExport ? defaultSettings : await detectSettings();
  const settings = defaultSettings;

  const nonce = headers().get('x-nonce') || '';

  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <InitColorSchemeScript {...schemeConfig} nonce={nonce} />
        <RoochDappProvider>
          <SettingsProvider settings={settings} caches="localStorage">
            <ThemeProvider nonce={nonce}>
              <MotionLazy>
                <Snackbar />
                <ProgressBar />
                <SettingsDrawer />
                <DashboardLayout>{children}</DashboardLayout>
              </MotionLazy>
            </ThemeProvider>
          </SettingsProvider>
        </RoochDappProvider>
      </body>
    </html>
  );
}
