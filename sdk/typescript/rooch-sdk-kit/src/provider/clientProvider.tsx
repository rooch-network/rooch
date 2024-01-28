// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useMemo, useState, createContext } from 'react'
import type { ReactNode } from 'react'

import { AllNetwork, Network, isRoochClient, RoochClient } from '@roochnetwork/rooch-sdk'

export interface RoochClientProviderContext {
  client: RoochClient
  networks: Network[]
  network: Network
  selectChain: (network: Network) => void
}

export const RoochClientContext = createContext<RoochClientProviderContext | null>(null)

export type RoochClientProviderProps<T extends Network> = {
  createClient?: (name: keyof T, config: T[keyof T]) => RoochClient
  children: ReactNode
  defaultNetwork: Network
  network?: Network[]
}

const DEFAULT_CREATE_CLIENT = function createClient(_name: string, config: Network | RoochClient) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

export function RoochClientProvider<T extends Network>(props: RoochClientProviderProps<T>) {
  const { defaultNetwork, children } = props

  const networks = props.network ?? AllNetwork

  const createClient = (props.createClient as typeof DEFAULT_CREATE_CLIENT) ?? DEFAULT_CREATE_CLIENT

  const [selectedNetwork, setSelectedNetwork] = useState<Network>(defaultNetwork ?? AllNetwork[0])

  const currentNetwork = props.defaultNetwork ?? selectedNetwork

  const client = useMemo(() => {
    return createClient(selectedNetwork.name, selectedNetwork)
  }, [createClient, selectedNetwork])

  const ctx = useMemo((): RoochClientProviderContext => {
    return {
      client,
      networks,
      network: currentNetwork,
      selectChain: (newNetwork) => {
        if (currentNetwork === newNetwork) {
          return
        }
        setSelectedNetwork(newNetwork)
      },
    }
  }, [client, networks, currentNetwork])

  return <RoochClientContext.Provider value={ctx}>{children}</RoochClientContext.Provider>
}
