// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createContext, useRef } from 'react'
import type { ReactNode } from 'react'
import type { StateStorage } from 'zustand/middleware'

import { createWalletStore, WalletStore } from '../walletStore'
import { useAutoConnectWallet } from '../hooks/wallet/useAutoConnectWallet'
import { ETHWallet } from '../types/ethWallet'

// TODO: use web3modal ?

type WalletProviderProps = {
  /** Enables automatically reconnecting to the most recently used wallet account upon mounting. */
  autoConnect?: boolean

  /** Configures how the most recently connected to wallet account is stored. Defaults to using localStorage. */
  storage?: StateStorage

  /** The key to use to store the most recently connected wallet account. */
  storageKey?: string

  children: ReactNode
}

const DEFAULT_STORAGE_KEY = 'rooch-sdk-kit:wallet-connect-info'

export const WalletContext = createContext<WalletStore | null>(null)

export function WalletProvider({
  storage = localStorage,
  storageKey = DEFAULT_STORAGE_KEY,
  autoConnect = false,
  children,
}: WalletProviderProps) {
  const storeRef = useRef(
    createWalletStore({
      wallet: new ETHWallet(),
      autoConnectEnabled: autoConnect,
      storage,
      storageKey,
    }),
  )

  return (
    // <MetaMaskProvider debug={true} sdkOptions={{
    //     logging: {developerMode: true},
    //     checkInstallationImmediately: autoConnect, // This will automatically connect to MetaMask on page load
    //     dappMetadata: {}
    // }}>
    <WalletContext.Provider value={storeRef.current}>
      <WalletConnectionManager>{children}</WalletConnectionManager>
    </WalletContext.Provider>
    // </MetaMaskProvider>
  )
}

type WalletConnectionManagerProps = Required<Pick<WalletProviderProps, 'children'>>

function WalletConnectionManager({ children }: WalletConnectionManagerProps) {
  useAutoConnectWallet()
  return children
}
