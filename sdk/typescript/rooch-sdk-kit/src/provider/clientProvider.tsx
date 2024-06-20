// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ReactNode } from 'react'

import { createContext, useMemo, useState } from 'react'

import {
  getRoochNodeUrl,
  isRoochClient,
  RoochClient,
  RoochClientOptions,
} from '@roochnetwork/rooch-sdk'
import type { NetworkConfig } from '@/hooks'
import { RoochSessionProvider } from './sessionProvider'

type NetworkConfigs<T extends NetworkConfig | RoochClient = NetworkConfig | RoochClient> = Record<
  string,
  T
>

export interface RoochClientProviderContext {
  client: RoochClient
  networks: NetworkConfigs
  network: string
  config: NetworkConfig | null
  selectNetwork: (network: string) => void
}

export const RoochClientContext = createContext<RoochClientProviderContext | null>(null)

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

const DEFAULT_CREATE_CLIENT = function createClient(
  _name: string,
  config: NetworkConfig | RoochClient,
) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

export function RoochClientProvider<T extends NetworkConfigs>(props: RoochClientProviderProps<T>) {
  const { onNetworkChange, network, children } = props
  const networks = (props.networks ?? DEFAULT_NETWORKS) as T
  const [selectedNetwork, setSelectedNetwork] = useState<keyof T & string>(
    props.network ?? props.defaultNetwork ?? (Object.keys(networks)[0] as keyof T & string),
  )
  const currentNetwork = props.network ?? selectedNetwork
  const createClient = DEFAULT_CREATE_CLIENT

  const client = useMemo(() => {
    return createClient(currentNetwork, networks[currentNetwork])
  }, [createClient, currentNetwork, networks])

  const ctx = useMemo((): RoochClientProviderContext => {
    return {
      client,
      network: currentNetwork,
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

  return (
    <RoochClientContext.Provider value={ctx}>
      <RoochSessionProvider> {children}</RoochSessionProvider>
    </RoochClientContext.Provider>
  )
}
