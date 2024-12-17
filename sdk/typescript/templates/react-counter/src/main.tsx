import React from 'react';
import ReactDOM from 'react-dom/client';
import "@roochnetwork/rooch-sdk-kit/dist/index.css";
import '@radix-ui/themes/styles.css';

import { RoochProvider, WalletProvider } from '@roochnetwork/rooch-sdk-kit'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {Theme} from '@radix-ui/themes';
import { networkConfig } from "./networks";
import App from './App';
import { ErrorGuard } from "./ErrorGuard.tsx";
import { DEVNET_COUNTER_PACKAGE_ID } from './constants.ts'

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme appearance="light">
      <QueryClientProvider client={queryClient}>
            <RoochProvider networks={networkConfig} sessionConf={
              {
                appName: "rooch_test",
                appUrl: "https://test.com",
                scopes: [`${DEVNET_COUNTER_PACKAGE_ID}::*::*`],
                maxInactiveInterval: 1200
              }
            } defaultNetwork='testnet'>
              <WalletProvider preferredWallets={['UniSat']} chain={'bitcoin'} autoConnect>
                <ErrorGuard/>
                <App/>
              </WalletProvider>
            </RoochProvider>
      </QueryClientProvider>
    </Theme>
  </React.StrictMode>
);
