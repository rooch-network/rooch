// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {createContext, useRef, useEffect, useState} from 'react'
import type { ReactNode } from 'react'

import {
  AllNetwork,
  Network,
  isRoochClient,
  RoochClient,
  RoochSessionAccount,
} from '@roochnetwork/rooch-sdk'
import { createClientStore } from '../clientStore'
import type { StateStorage } from 'zustand/middleware'
import { createSessionStore } from '../sessionStore'

const DEFAULT_STORAGE_KEY = 'rooch-sdk-kit:rooch-client-info'
const DEFAULT_SESSION_STORAGE_KEY = 'rooch-sdk-kit:rooch-session-info'

export type RoochClientProviderContextActions = {
  addNetwork: (network: Network) => void
  switchNetwork: (network: Network) => void
  removeNetwork: (network: Network) => void
  addSession: (session: RoochSessionAccount) => void
  setCurrentSession: (session: RoochSessionAccount) => void
  removeSession: (session: RoochSessionAccount) => void
}

export type RoochClientProviderContext = {
  client: RoochClient
  networks: Network[]
  currentNetwork: Network
  sessions: RoochSessionAccount[]
  currentSession: RoochSessionAccount | null
} & RoochClientProviderContextActions

export const RoochClientContext = createContext<RoochClientProviderContext | null>(null)

export type RoochClientProviderProps<T extends Network> = {
  network: Network
  networks?: Network[]
  createClient?: (name: keyof T, config: T[keyof T]) => RoochClient
  /** Configures how the most recently connected to wallet account is stored. Defaults to using localStorage. */
  storage?: StateStorage

  /** The key to use to store the most recently connected wallet account. */
  storageKey?: string

  children: ReactNode
}

const DEFAULT_CREATE_CLIENT = function createClient(_name: string, config: Network | RoochClient) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

export function RoochClientProvider<T extends Network>(props: RoochClientProviderProps<T>) {
  // ** Props **
  const { storage, storageKey, network, networks, children } = props

  // ** Hooks **
  const createClient = (props.createClient as typeof DEFAULT_CREATE_CLIENT) ?? DEFAULT_CREATE_CLIENT

  const [initializing, setInitializing] = useState(true)

  const clientStoreRef = useRef<ReturnType<typeof createClientStore>>()
  const sessionStoreRef = useRef<ReturnType<typeof createSessionStore>>()
  const clientRef = useRef<RoochClient>()
  const ctxRef = useRef<RoochClientProviderContext>()

  // ** Init **
  useEffect(() => {
    // client store
    clientStoreRef.current = createClientStore({
      storage: storage ?? localStorage,
      storageKey: storageKey ?? DEFAULT_STORAGE_KEY,
      networks: networks ?? AllNetwork,
      currentNetwork: network,
    })

    // session store
    sessionStoreRef.current = createSessionStore({
      storage: sessionStorage,
      storageKey: DEFAULT_SESSION_STORAGE_KEY,
    })

    {
      const { currentNetwork } = clientStoreRef.current!.getState()
      clientRef.current = createClient(currentNetwork.name, currentNetwork)

      const { networks, addNetwork, switchNetwork, removeNetwork } =
        clientStoreRef.current!.getState()
      const { sessions, currentSession, addSession, setCurrentSession, removeSession } =
        sessionStoreRef.current!.getState()
      ctxRef.current = {
        client: clientRef.current!,
        networks: networks,
        currentNetwork: currentNetwork,
        sessions,
        currentSession,
        addNetwork,
        switchNetwork,
        removeNetwork,
        addSession,
        setCurrentSession,
        removeSession,
      }

      setInitializing(false)
      console.log('net', clientStoreRef.current.getState())
      console.log('init fin', ctxRef.current)
    }
  }, [createClient, network, networks, storage, storageKey])

  console.log(clientRef.current)

  return initializing ? (
    <></>
  ) : (
    <RoochClientContext.Provider value={ctxRef.current!}>{children}</RoochClientContext.Provider>
  )
}
