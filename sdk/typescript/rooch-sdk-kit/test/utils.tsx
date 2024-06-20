// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ComponentProps } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { RoochClient, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { RoochClientProvider } from '../src/provider/clientProvider.js'
import { WalletProvider } from '../src/provider/walletProvider.js'

export function createRoochClientContextWrapper(client: RoochClient) {
  return function RoochClientContextWrapper({ children }: { children: React.ReactNode }) {
    return <RoochClientProvider networks={{ test: client }}>{children}</RoochClientProvider>
  }
}

export function createWalletProviderContextWrapper(
  providerProps: Omit<ComponentProps<typeof WalletProvider>, 'children'> = {},
  suiClient: RoochClient = new RoochClient({ url: getRoochNodeUrl('localnet') }),
) {
  const queryClient = new QueryClient()
  return function WalletProviderContextWrapper({ children }: { children: React.ReactNode }) {
    return (
      <RoochClientProvider networks={{ test: suiClient }}>
        <QueryClientProvider client={queryClient}>
          <WalletProvider {...providerProps}>{children}</WalletProvider>;
        </QueryClientProvider>
      </RoochClientProvider>
    )
  }
}
