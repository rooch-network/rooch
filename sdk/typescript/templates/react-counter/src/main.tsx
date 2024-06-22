// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react';
import ReactDOM from 'react-dom/client';
import '@radix-ui/themes/styles.css';

import {RoochClientProvider, SupportChain, WalletProvider} from '@roochnetwork/rooch-sdk-kit';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {Theme} from '@radix-ui/themes';
import { networkConfig } from "./networks.ts";
import App from './App';

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme appearance="dark">
      <QueryClientProvider client={queryClient}>
            <RoochClientProvider networks={networkConfig} defaultNetwork="localnet">
                <WalletProvider chain={ SupportChain.BITCOIN } autoConnect>
                  <App/>
                </WalletProvider>
            </RoochClientProvider>
      </QueryClientProvider>
    </Theme>
  </React.StrictMode>
);
