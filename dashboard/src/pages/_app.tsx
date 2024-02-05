// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Next Imports
import Head from 'next/head'
import { Router } from 'next/router'
import type { NextPage } from 'next'
import type { AppProps } from 'next/app'

// ** Loader Import
import NProgress from 'nprogress'

// ** Emotion Imports
import { CacheProvider } from '@emotion/react'
import type { EmotionCache } from '@emotion/cache'

// ** Config Imports

import themeConfig from 'src/configs/themeConfig'

// ** Third Party Import
import { Toaster } from 'react-hot-toast'

// ** Component Imports
import UserLayout from 'src/layouts/UserLayout'
import ThemeComponent from 'src/@core/theme/ThemeComponent'

// ** Contexts
import { SettingsConsumer, SettingsProvider } from 'src/@core/context/settingsContext'

// ** Styled Components
import ReactHotToast from 'src/@core/styles/libs/react-hot-toast'

// ** Utils Imports
import { createEmotionCache } from 'src/@core/utils/create-emotion-cache'

// ** Prismjs Styles
import 'prismjs'
import 'prismjs/themes/prism-tomorrow.css'
import 'prismjs/components/prism-jsx'
import 'prismjs/components/prism-tsx'

// ** React Perfect Scrollbar Style
import 'react-perfect-scrollbar/dist/css/styles.css'

import 'src/iconify-bundle/icons-bundle-react'

// ** Global css styles
import '../../styles/globals.css'

import { GoogleAnalytics } from 'nextjs-google-analytics'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { TESTNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'
import Spinner from 'src/@core/components/spinner'

// import SessionGuard from 'src/auth/sessionGuard'

// ** Extend App Props with Emotion
type ExtendedAppProps = AppProps & {
  Component: NextPage
  emotionCache: EmotionCache
}

// type GuardProps = {
//   authGuard: boolean
//   children: ReactNode
// }

const clientSideEmotionCache = createEmotionCache()

// ** Pace Loader
if (themeConfig.routingLoader) {
  Router.events.on('routeChangeStart', () => {
    NProgress.start()
  })
  Router.events.on('routeChangeError', () => {
    NProgress.done()
  })
  Router.events.on('routeChangeComplete', () => {
    NProgress.done()
  })
}

// const Guard = ({ children, authGuard }: GuardProps) => {
//   return authGuard ? <SessionGuard>{children}</SessionGuard> : children
// }

// ** Configure JSS & ClassName
const App = (props: ExtendedAppProps) => {
  const { Component, emotionCache = clientSideEmotionCache, pageProps } = props

  // Variables
  const contentHeightFixed = Component.contentHeightFixed ?? false
  const getLayout =
    Component.getLayout ??
    ((page) => <UserLayout contentHeightFixed={contentHeightFixed}>{page}</UserLayout>)

  const setConfig = Component.setConfig ?? undefined

  const queryClient = new QueryClient()

  return (
    <CacheProvider value={emotionCache}>
      <Head>
        <title>{`${themeConfig.templateName} - Dashboard`}</title>
        <meta name="description" content={`${themeConfig.templateName} – Dashboard`} />
        <meta name="keywords" content="Rooch, Dashboard" />
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <QueryClientProvider client={queryClient}>
        <RoochClientProvider defaultNetwork={TESTNetwork}>
          <WalletProvider chain={SupportChain.BITCOIN} autoConnect fallback={<Spinner />}>
            <SettingsProvider {...(setConfig ? { pageSettings: setConfig() } : {})}>
              <SettingsConsumer>
                {({ settings }) => {
                  return (
                    <ThemeComponent settings={settings}>
                      {/*<Guard authGuard={true}>*/}
                      {getLayout(
                        <>
                          <GoogleAnalytics trackPageViews></GoogleAnalytics>
                          <Component {...pageProps} />
                        </>,
                      )}
                      {/*</Guard>*/}
                      <ReactHotToast>
                        <Toaster
                          position={settings.toastPosition}
                          toastOptions={{ className: 'react-hot-toast' }}
                        />
                      </ReactHotToast>
                    </ThemeComponent>
                  )
                }}
              </SettingsConsumer>
            </SettingsProvider>
          </WalletProvider>
        </RoochClientProvider>
      </QueryClientProvider>
    </CacheProvider>
  )
}

export default App
