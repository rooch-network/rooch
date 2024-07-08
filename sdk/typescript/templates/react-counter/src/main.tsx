import React from 'react';
import ReactDOM from 'react-dom/client';
import '@radix-ui/themes/styles.css';

import { RoochProvider, WalletProvider } from '@roochnetwork/rooch-sdk-kit'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {Theme} from '@radix-ui/themes';
import { networkConfig } from "./networks";
import App from './App';

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme appearance="dark">
      <QueryClientProvider client={queryClient}>
            <RoochProvider networks={networkConfig} defaultNetwork='localnet'>
              <WalletProvider preferredWallets={['onekey-hardware']} chain={'bitcoin'} autoConnect>
                <App/>
              </WalletProvider>
            </RoochProvider>
      </QueryClientProvider>
    </Theme>
  </React.StrictMode>
);
