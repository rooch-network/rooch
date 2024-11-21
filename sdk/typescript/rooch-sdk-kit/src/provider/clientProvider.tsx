// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode } from 'react'

import { createContext, useMemo, useState } from 'react'

import {
  getRoochNodeUrl,
  RoochClient,
  RoochClientOptions,
  NetworkType,
} from '@roochnetwork/rooch-sdk'

import { NetworkConfig } from '../hooks/index.js'
import { useDefaultClient } from './useDefaultClient.js'

export type NetworkConfigs<T extends NetworkConfig | RoochClient = NetworkConfig | RoochClient> =
  Record<string, T>

export interface ClientProviderContext {
  client: RoochClient
  networks: NetworkConfigs
  network: NetworkType
  config: NetworkConfig | null
  selectNetwork: (network: string) => void
}

export const ClientContext = createContext<ClientProviderContext | null>(null)

export type RoochClientProviderProps<T extends NetworkConfigs> = {
  networks?: NetworkConfigs
  onNetworkChange?: (network: keyof T & string) => void
  children: ReactNode
} & (
  | {
      defaultNetwork?: keyof T & string
      network?: never
    }
  | {
      defaultNetwork?: never
      network?: keyof T & string
    }
)

const DEFAULT_NETWORKS = {
  localnet: { url: getRoochNodeUrl('localnet') },
}

export function RoochClientProvider<T extends NetworkConfigs>(props: RoochClientProviderProps<T>) {
  const { onNetworkChange, network, children } = props
  const networks = (props.networks ?? DEFAULT_NETWORKS) as T
  const [selectedNetwork, setSelectedNetwork] = useState<keyof T & string>(
    props.network ?? props.defaultNetwork ?? (Object.keys(networks)[0] as keyof T & string),
  )
  const currentNetwork = props.network ?? selectedNetwork

  const client = useDefaultClient({ currentNetwork, networks })

  const ctx = useMemo((): ClientProviderContext => {
    return {
      client,
      network: currentNetwork as NetworkType,
      networks,
      config:
        networks[currentNetwork] instanceof RoochClient
          ? null
          : (networks[currentNetwork] as RoochClientOptions),
      selectNetwork: (newNetwork) => {
        if (currentNetwork === newNetwork) {
          return
        }

        if (!network && newNetwork !== selectedNetwork) {
          setSelectedNetwork(newNetwork)
        }

        onNetworkChange?.(newNetwork)
      },
    }
  }, [client, currentNetwork, networks, network, selectedNetwork, onNetworkChange])

  return <ClientContext.Provider value={ctx}>{children}</ClientContext.Provider>
}
