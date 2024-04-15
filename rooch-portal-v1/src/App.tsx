import { useEffect, useState } from 'react'

import { CacheProvider } from '@emotion/react'
import { Toaster } from '@/components/ui/toaster'
import { ThemeProvider } from '@/components/theme-provider'
import { SessionKeyModal } from '@/components/session-key-modal'

import { createEmotionCache } from '@/utils/create-emotion-cache'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { TestNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'

import { DashboardLayout } from '@/pages/dashboard-layout'
import { ToastProvider } from '@/providers/toast-provider'

const clientSideEmotionCache = createEmotionCache()

function App() {
  const queryClient = new QueryClient()
  const [isSessionKeyModalOpen, setIsSessionKeyModalOpen] = useState<boolean>(false)
  const handleSessionKeyRequest = () => {
    setIsSessionKeyModalOpen(true)
  }

  const handleAuthorize = () => {
    console.log('Handling authorization in App component.')

    setIsSessionKeyModalOpen(false)
  }

  // ** Session Key Modal 测试（可以关掉）
  useEffect(() => {
    handleSessionKeyRequest()
  }, [])

  return (
    <>
      <CacheProvider value={clientSideEmotionCache}>
        <QueryClientProvider client={queryClient}>
          <RoochClientProvider network={TestNetwork}>
            <WalletProvider chain={SupportChain.BITCOIN} autoConnect>
              <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
                <ToastProvider />
                <DashboardLayout />
                <SessionKeyModal />
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
