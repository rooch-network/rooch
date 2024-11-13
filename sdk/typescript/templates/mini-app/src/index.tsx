import ReactDOM from 'react-dom/client'
import { StrictMode } from 'react'
import { retrieveLaunchParams } from '@telegram-apps/sdk-react'

import { Root } from '@/components/Root.tsx'
import { EnvUnsupported } from '@/components/EnvUnsupported.tsx'
import { init } from '@/init.ts'

import '@telegram-apps/telegram-ui/dist/styles.css'
import './index.css'

// Mock the environment in case, we are outside Telegram.
import './mockEnv.ts'
import { TonConnectUIProvider } from '@tonconnect/ui-react'

const root = ReactDOM.createRoot(document.getElementById('root')!)

try {
  // Configure all application dependencies.
  init(retrieveLaunchParams().startParam === 'debug' || import.meta.env.DEV)

  root.render(
    <StrictMode>
      <TonConnectUIProvider
        manifestUrl="https://ton-connect.github.io/demo-dapp-with-wallet/tonconnect-manifest.json"
        walletsListConfiguration={{
          includeWallets: [
            {
              appName: 'tonwallet',
              name: 'TON Wallet',
              imageUrl: 'https://wallet.ton.org/assets/ui/qr-logo.png',
              aboutUrl:
                'https://chrome.google.com/webstore/detail/ton-wallet/nphplpgoakhhjchkkhmiggakijnkhfnd',
              universalLink: 'https://wallet.ton.org/ton-connect',
              jsBridgeKey: 'tonwallet',
              bridgeUrl: 'https://bridge.tonapi.io/bridge',
              platforms: ['chrome', 'android'],
            },
          ],
        }}
      >
        <Root />
      </TonConnectUIProvider>
    </StrictMode>,
  )
} catch (e) {
  root.render(<EnvUnsupported />)
}
