// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { CacheProvider } from '@emotion/react'
import { Toaster } from '@/components/ui/toaster'
import { ThemeProvider } from '@/components/theme-provider'

import { createEmotionCache } from '@/utils/create-emotion-cache'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { WalletProvider, RoochProvider } from '@roochnetwork/rooch-sdk-kit'

import { DashboardLayout } from '@/pages/dashboard-layout'
import { ToastProvider } from '@/providers/toast-provider'
import { SessionGuard } from '@/guard/session.tsx'
import { networkConfig } from '@/networks'

const clientSideEmotionCache = createEmotionCache()

function App() {
  const queryClient = new QueryClient()

  return (
    <>
      <CacheProvider value={clientSideEmotionCache}>
        <QueryClientProvider client={queryClient}>
          <RoochProvider networks={networkConfig} defaultNetwork="testnet">
            <WalletProvider chain={'bitcoin'} autoConnect>
              <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
                <ToastProvider />
                <SessionGuard>
                  <DashboardLayout />
                </SessionGuard>
              </ThemeProvider>
            </WalletProvider>
            <Toaster />
          </RoochProvider>
        </QueryClientProvider>
      </CacheProvider>
    </>
  )
}

export default App
