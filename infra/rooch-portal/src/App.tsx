// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { CacheProvider } from '@emotion/react'
import { Toaster } from '@/components/ui/toaster'
import { ThemeProvider } from '@/components/theme-provider'

import { createEmotionCache } from '@/utils/create-emotion-cache'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { TestNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'

import { DashboardLayout } from '@/pages/dashboard-layout'
import { ToastProvider } from '@/providers/toast-provider'
import { SessionGuard } from '@/guard/session.tsx'

const clientSideEmotionCache = createEmotionCache()

function App() {
  const queryClient = new QueryClient()

  return (
    <>
      <CacheProvider value={clientSideEmotionCache}>
        <QueryClientProvider client={queryClient}>
          <RoochClientProvider network={TestNetwork}>
            <WalletProvider
              chain={SupportChain.BITCOIN}
              autoConnect
              fallback={
                <div className="h-screen w-screen flex items-center justify-center bg-gradient-to-r from-zinc-900 to-zinc-800">
                  <div className="text-center">
                    <div className="flex items-center justify-center mb-6">
                      <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
                    </div>
                    <h3 className="text-white text-2xl font-semibold">Loading data...</h3>
                    <p className="text-gray-400 mt-2">
                      Please wait a moment while we fetch your data.
                    </p>
                  </div>
                </div>
              }
            >
              <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
                <ToastProvider />
                <SessionGuard>
                  <DashboardLayout />
                </SessionGuard>
              </ThemeProvider>
            </WalletProvider>
            <Toaster />
          </RoochClientProvider>
        </QueryClientProvider>
      </CacheProvider>
    </>
  )
}

export default App
