// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createContext, useMemo, useRef } from 'react'
import type { ReactNode } from 'react'

import { AllNetwork, isRoochClient, Network, RoochClient } from '@roochnetwork/rooch-sdk'
import { ClientContextStore, createClientContextStore } from '../clientStore'
import type { StateStorage } from 'zustand/middleware'
import { useCurrentNetwork } from '../hooks'
import { RoochSessionProvider } from './sessionProvider'
import { getDefaultStorage, StorageType } from '../utils/stateStorage'

const DEFAULT_STORAGE_KEY = 'rooch-sdk-kit:rooch-client-info'

export const RoochClientContext = createContext<ClientContextStore | null>(null)

export type RoochClientProviderProps = {
  network: Network
  networks?: Network[]
  /** Configures how the most recently connected to wallet account is stored. Defaults to using localStorage. */
  storage?: StateStorage

  /** The key to use to store the most recently connected wallet account. */
  storageKey?: string

  children: ReactNode
}

export function RoochClientProvider(props: RoochClientProviderProps) {
  // ** Props **
  const { storage, storageKey, network, networks, children } = props

  const clientStoreRef = useRef(
    createClientContextStore({
      storage: storage || getDefaultStorage(StorageType.Local),
      storageKey: storageKey || DEFAULT_STORAGE_KEY + network.id,
      networks: networks || AllNetwork,
      currentNetwork: network,
    }),
  )

  return (
    <RoochClientContext.Provider value={clientStoreRef.current}>
      <ClientManagerProvider> {children} </ClientManagerProvider>
    </RoochClientContext.Provider>
  )
}

export const ClientManagerContext = createContext<RoochClient | null>(null)

type ClientManagerProps = Required<Pick<RoochClientProviderProps, 'children'>>

const DEFAULT_CREATE_CLIENT = function createClient(config: Network | RoochClient) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

function ClientManagerProvider(props: ClientManagerProps) {
  // ** Props **
  const { children } = props

  // ** Hooks **
  const network = useCurrentNetwork()
  const createClient = DEFAULT_CREATE_CLIENT

  const client = useMemo(() => {
    return createClient(network)
  }, [createClient, network])

  return (
    <ClientManagerContext.Provider value={client}>
      <RoochSessionProvider> {children}</RoochSessionProvider>
    </ClientManagerContext.Provider>
  )
}
