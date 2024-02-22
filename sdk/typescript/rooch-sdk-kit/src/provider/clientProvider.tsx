// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useMemo, createContext, useRef, useEffect, useState } from 'react'
import type { ReactNode } from 'react'

import { AllNetwork, Network, isRoochClient, RoochClient } from '@roochnetwork/rooch-sdk'
import { createClientStore } from '../clientStore'
import type { StateStorage } from 'zustand/middleware'

const DEFAULT_STORAGE_KEY = 'rooch-sdk-kit:rooch-client-info'

export interface RoochClientProviderContext {
  client: RoochClient
  supportNetworks: Network[]
  currentNetwork: Network
  selectedNetwork: (network: Network) => void
  addNetwork: (network: Network) => void
}

export const RoochClientContext = createContext<RoochClientProviderContext | null>(null)

export type RoochClientProviderProps<T extends Network> = {
  defaultNetwork: Network
  children: ReactNode
  supportNetworks?: Network[]
  createClient?: (name: keyof T, config: T[keyof T]) => RoochClient
  /** Configures how the most recently connected to wallet account is stored. Defaults to using localStorage. */
  storage?: StateStorage

  /** The key to use to store the most recently connected wallet account. */
  storageKey?: string
}

const DEFAULT_CREATE_CLIENT = function createClient(_name: string, config: Network | RoochClient) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

export function RoochClientProvider<T extends Network>(props: RoochClientProviderProps<T>) {
  const { storage, storageKey, defaultNetwork, supportNetworks, children } = props
  const createClient = (props.createClient as typeof DEFAULT_CREATE_CLIENT) ?? DEFAULT_CREATE_CLIENT

  const [initializing, setInitializing] = useState(true)
  const [currentNetwork, setCurrentNetwork] = useState<Network>(defaultNetwork)

  const storeRef = useRef<ReturnType<typeof createClientStore>>()
  useEffect(() => {
    storeRef.current = createClientStore({
      storage: storage ?? localStorage,
      storageKey: storageKey ?? DEFAULT_STORAGE_KEY,
    })

    const { lastConnectedNetwork, setLastConnectedNetwork } = storeRef.current!.getState()
    if (lastConnectedNetwork) {
      setCurrentNetwork(lastConnectedNetwork)
    } else {
      setLastConnectedNetwork(defaultNetwork)
    }

    setInitializing(false)
    console.log(storeRef.current)
  }, [defaultNetwork, storage, storageKey])

  const networks = useMemo(() => {
    const networks = supportNetworks ?? AllNetwork
    const store = storeRef.current?.getState()
    if (store) {
      const { networks: customNetworks } = store
      return networks.concat(customNetworks)
    }
    return networks
  }, [storeRef, supportNetworks])

  const client = useMemo(() => {
    return createClient(currentNetwork.name, currentNetwork)
  }, [createClient, currentNetwork])

  const ctx = useMemo((): RoochClientProviderContext => {
    return {
      client,
      supportNetworks: networks,
      currentNetwork: currentNetwork,
      selectedNetwork: (newNetwork) => {
        if (currentNetwork === newNetwork) {
          return
        }
        setCurrentNetwork(newNetwork)
      },
      addNetwork: (network) => {
        const { addNetwork } = storeRef.current!.getState()
        addNetwork(network)
      },
    }
  }, [networks, client, currentNetwork])

  return initializing ? (
    <></>
  ) : (
    <RoochClientContext.Provider value={ctx}>{children}</RoochClientContext.Provider>
  )
}
