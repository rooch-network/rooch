// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ComponentProps } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RoochClient, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

import { RoochProvider, WalletProvider } from '../src/provider/index.js'
import { MockBitcoinWallet } from './mocks/mock-wallet.js'
import { getWallets } from '../src/wellet/wallets.js'

export function createRoochClientContextWrapper(client: RoochClient) {
  return function RoochClientContextWrapper({ children }: { children: React.ReactNode }) {
    return <RoochProvider networks={{ test: client }}>{children}</RoochProvider>
  }
}

export function createWalletProviderContextWrapper(
  providerProps: Omit<ComponentProps<typeof WalletProvider>, 'children'> = {},
  client: RoochClient = new RoochClient({ url: getRoochNodeUrl('localnet') }),
) {
  const queryClient = new QueryClient()
  return function WalletProviderContextWrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <RoochProvider networks={{ test: client }}>
          <WalletProvider {...providerProps}>{children}</WalletProvider>;
        </RoochProvider>
      </QueryClientProvider>
    )
  }
}

export function registerMockWallet() {
  const mockWallet = new MockBitcoinWallet()
  return {
    mockWallet,
    unregister: getWallets().register(mockWallet),
  }
}
