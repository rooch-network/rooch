import { CacheProvider } from '@emotion/react'
import { ThemeProvider } from '@/components/theme-provider'
import { Toaster } from '@/components/ui/toaster'

import { createEmotionCache } from '@/utils/create-emotion-cache'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { TestNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'

import { DashboardLayout } from './pages/dashboard-layout'
import { ToastProvider } from './providers/toast-provider'
// import { Banner } from './components/banner'

const clientSideEmotionCache = createEmotionCache()

function App() {
  const queryClient = new QueryClient()

  return (
    <>
      <CacheProvider value={clientSideEmotionCache}>
        <QueryClientProvider client={queryClient}>
          <RoochClientProvider network={TestNetwork}>
            <WalletProvider chain={SupportChain.BITCOIN} autoConnect>
              <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
                <ToastProvider />
                {/* <Banner /> */}
                <DashboardLayout />
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
