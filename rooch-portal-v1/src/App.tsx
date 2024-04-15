import { CacheProvider } from '@emotion/react'
import { ThemeProvider } from '@/components/theme-provider'
import { Toaster } from '@/components/ui/toaster'

import { createEmotionCache } from '@/utils/create-emotion-cache'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { TestNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'

import { DashboardLayout } from './pages/dashboard-layout'
import { ToastProvider } from './providers/toast-provider'
import { useEffect, useState } from 'react'

import SessionKeyModal from '@/components/session-key-modal'

const clientSideEmotionCache = createEmotionCache()

const App = () => {
  const queryClient = new QueryClient()
  const [isSessionKeyModalOpen, setIsSessionKeyModalOpen] = useState<boolean>(false)
  const handleSessionKeyRequest = () => {
    setIsSessionKeyModalOpen(true)
  }

  const handleAuthorize = () => {
    console.log('Handling authorization in App component.')

    setIsSessionKeyModalOpen(false)
  }

  // 如果要测试 Session Key Modal，打开这个就行
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
                <SessionKeyModal
                  isOpen={isSessionKeyModalOpen}
                  onClose={() => setIsSessionKeyModalOpen(false)}
                  onAuthorize={handleAuthorize}
                />
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
