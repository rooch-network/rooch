// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useMemo, useState, createContext } from 'react'
import type { ReactNode } from 'react'

import { AllChain, Chain, isRoochClient, RoochClient } from '@roochnetwork/rooch-sdk'

export interface RoochClientProviderContext {
  client: RoochClient
  chains: Chain[]
  chain: Chain
  selectChain: (network: Chain) => void
}

export const RoochClientContext = createContext<RoochClientProviderContext | null>(null)

export type RoochClientProviderProps<T extends Chain> = {
  createClient?: (name: keyof T, config: T[keyof T]) => RoochClient
  children: ReactNode
  defaultChain: Chain
  chains?: Chain[]
}

const DEFAULT_CREATE_CLIENT = function createClient(_name: string, config: Chain | RoochClient) {
  if (isRoochClient(config)) {
    return config
  }

  return new RoochClient(config)
}

export function RoochClientProvider<T extends Chain>(props: RoochClientProviderProps<T>) {
  const { defaultChain, children } = props

  const chains = props.chains ?? AllChain

  const createClient = (props.createClient as typeof DEFAULT_CREATE_CLIENT) ?? DEFAULT_CREATE_CLIENT

  const [selectedChain, setSelectedChain] = useState<Chain>(defaultChain ?? AllChain[0])

  const currentChain = props.defaultChain ?? selectedChain

  const client = useMemo(() => {
    return createClient(selectedChain.name, selectedChain)
  }, [createClient, selectedChain])

  const ctx = useMemo((): RoochClientProviderContext => {
    return {
      client,
      chains,
      chain: currentChain,
      selectChain: (newChain) => {
        if (currentChain === newChain) {
          return
        }
        setSelectedChain(newChain)
      },
    }
  }, [client, chains, currentChain])

  return <RoochClientContext.Provider value={ctx}>{children}</RoochClientContext.Provider>
}
