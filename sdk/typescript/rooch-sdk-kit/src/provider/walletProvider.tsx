// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createContext, useEffect, useRef, useState, ReactNode } from 'react'
import type { StateStorage } from 'zustand/middleware'

import { createWalletStore, WalletStore } from '../walletStore'
import { useAutoConnectWallet } from '../hooks'
import { getInstalledWallets } from '../utils/walletUtils'
import { BaseWallet } from '../types/wellet/baseWallet'
import { SupportWallet } from '../feature'

// TODO: use web3modal ?

type WalletProviderProps = {
  wallet?: SupportWallet

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
  wallet = SupportWallet.ETH,
  storage = localStorage,
  storageKey = DEFAULT_STORAGE_KEY,
  autoConnect = false,
  children,
}: WalletProviderProps) {
  const [wallets, setWallets] = useState<BaseWallet[]>()
  const [loading, setLoading] = useState(true)
  const storeRef = useRef<ReturnType<typeof createWalletStore>>()

  useEffect(() => {
    getInstalledWallets(wallet).then((v) => setWallets(v))
  }, [wallet])

  useEffect(() => {
    if (wallets && wallets.length !== 0) {
      storeRef.current = createWalletStore({
        wallet: wallets[0],
        autoConnectEnabled: autoConnect,
        storage,
        storageKey,
      })
      setLoading(false)
    }
  }, [wallets, autoConnect, storageKey, storage])

  // TODO: how to show loading ?
  return !loading ? (
    <WalletContext.Provider value={storeRef.current!}>
      <WalletConnectionManager>{children}</WalletConnectionManager>
    </WalletContext.Provider>
  ) : (
    <>loading</>
  )
}

type WalletConnectionManagerProps = Required<Pick<WalletProviderProps, 'children'>>

function WalletConnectionManager({ children }: WalletConnectionManagerProps) {
  useAutoConnectWallet()
  return children
}
