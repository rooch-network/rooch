// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ComponentProps } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RoochClient, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

import { RoochClientProvider, WalletProvider } from '../src/provider/index.js'

export function createRoochClientContextWrapper(client: RoochClient) {
  return function RoochClientContextWrapper({ children }: { children: React.ReactNode }) {
    return <RoochClientProvider networks={{ test: client }}>{children}</RoochClientProvider>
  }
}

export function createWalletProviderContextWrapper(
  providerProps: Omit<ComponentProps<typeof WalletProvider>, 'children'> = {},
  roochClient: RoochClient = new RoochClient({ url: getRoochNodeUrl('localnet') }),
) {
  const queryClient = new QueryClient()
  return function WalletProviderContextWrapper({ children }: { children: React.ReactNode }) {
    return (
      <RoochClientProvider networks={{ test: roochClient }}>
        <QueryClientProvider client={queryClient}>
          <WalletProvider {...providerProps}>{children}</WalletProvider>;
        </QueryClientProvider>
      </RoochClientProvider>
    )
  }
}
