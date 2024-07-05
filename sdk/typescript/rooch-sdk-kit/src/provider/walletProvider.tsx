// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode, useCallback, createContext, useEffect, useRef } from 'react'
import type { StateStorage } from 'zustand/middleware'
import { BitcoinAddress, Session } from '@roochnetwork/rooch-sdk'

import { createWalletStore, WalletStore } from './walletStore.js'
import {
  useAutoConnectWallet,
  useCurrentSession,
  useSession,
  useWalletStore,
  useCurrentNetwork,
} from '../hooks/index.js'
import { useSessionStore } from '../hooks/useSessionsStore.js'
import { getDefaultStorage, StorageType, checkWallets } from '../utils/index.js'
import { SupportChain, SupportWallet } from '../feature/index.js'
import { getRegisteredWallets } from '../wellet/util.js'
import { getWallets } from '../wellet/wallets.js'
import { useWalletChanged } from '../hooks/index.js'

type WalletProviderProps = {
  preferredWallets?: SupportWallet[]

  chain?: SupportChain

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
  preferredWallets = ['unisat', 'okx'],
  chain = 'bitcoin',
  storage,
  storageKey = DEFAULT_STORAGE_KEY,
  autoConnect = false,
  children,
}: WalletProviderProps) {
  const network = useCurrentNetwork()

  const storeRef = useRef(
    createWalletStore({
      chain,
      wallets: getRegisteredWallets(preferredWallets, (w) => w.getChain() === chain),
      currentWallet: undefined,
      autoConnectEnabled: autoConnect,
      storage: storage || getDefaultStorage(StorageType.Local),
      storageKey: storageKey + network + chain?.toString(),
    }),
  )

  useEffect(() => {
    const fetchWallet = async () => {
      const wallets = await checkWallets(chain)
      getWallets().register(...wallets)
    }

    fetchWallet()
  }, [chain])

  return (
    <WalletContext.Provider value={storeRef.current}>
      <WalletConnectionManager preferredWallets={preferredWallets} chain={chain}>
        {children}
      </WalletConnectionManager>
    </WalletContext.Provider>
  )
}

type WalletConnectionManagerProps = Required<
  Pick<WalletProviderProps, 'children' | 'preferredWallets' | 'chain'>
>

function WalletConnectionManager({ children, preferredWallets }: WalletConnectionManagerProps) {
  useAutoConnectWallet()
  useWalletChanged(preferredWallets)

  const connectionStatus = useWalletStore((store) => store.connectionStatus)
  const currentWallet = useWalletStore((store) => store.currentWallet)
  const setWalletDisconnected = useWalletStore((store) => store.setWalletDisconnected)
  const setConnectionStatus = useWalletStore((state) => state.setConnectionStatus)
  const setAddressSwitched = useWalletStore((store) => store.setAddressSwitched)
  const currentAddress = useWalletStore((state) => state.currentAddress)
  const sessions = useSession()
  const curSession = useCurrentSession()
  const setCurrentSession = useSessionStore((state) => state.setCurrentSession)

  const accountsChangedHandler = useCallback(
    async (address: string[]) => {
      if (address.length === 0) {
        setWalletDisconnected()
      } else {
        setConnectionStatus('connecting')
        const selectedAddress = address[0]
        if (selectedAddress !== currentAddress?.toStr()) {
          setAddressSwitched(new BitcoinAddress(selectedAddress))
          setCurrentSession(undefined)
          const cur = sessions.find(
            (item: Session) =>
              item.getRoochAddress().toStr() === currentAddress?.genRoochAddress().toStr(),
          )
          if (cur && cur.getAuthKey() !== curSession?.getAuthKey()) {
            setCurrentSession(cur)
          }
        }
      }
    },
    [
      sessions,
      curSession,
      currentAddress,
      setAddressSwitched,
      setConnectionStatus,
      setCurrentSession,
      setWalletDisconnected,
    ],
  )

  // handle Listener
  useEffect(() => {
    if (connectionStatus === 'connected') {
      currentWallet?.onAccountsChanged(accountsChangedHandler)
    }

    return () => {
      if (connectionStatus === 'connected') {
        currentWallet?.removeAccountsChanged(accountsChangedHandler)
      }
    }
  }, [accountsChangedHandler, connectionStatus, currentWallet])

  return children
}
