// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { Network } from '@roochnetwork/rooch-sdk'

export type ClientContextActions = {
  addNetwork: (network: Network) => void
  switchNetwork: (network: Network) => void
  removeNetwork: (network: Network) => void
}

export type ClientContextStoreState = {
  networks: Network[]
  currentNetwork: Network
} & ClientContextActions

export type ClientContextStore = ReturnType<typeof createClientContextStore>

type ClientContextConfiguration = {
  storage: StateStorage
  storageKey: string
  networks: Network[]
  currentNetwork: Network
}

export function createClientContextStore({
  storage,
  storageKey,
  currentNetwork,
}: ClientContextConfiguration) {
  return createStore<ClientContextStoreState>()(
    persist(
      (set, get) => ({
        networks: [],
        currentNetwork: currentNetwork,
        addNetwork(network) {
          const cache = get().networks
          set(() => ({
            networks: cache.concat(network),
          }))
        },
        removeNetwork(network) {
          const cache = get().networks
          const networks = cache.filter((item) => item.id !== network.id)
          set(() => ({
            networks: networks,
          }))
        },
        switchNetwork(network) {
          set(() => ({
            currentNetwork: network,
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ networks, currentNetwork }) => ({
          networks,
          currentNetwork,
        }),
      },
    ),
  )
}
