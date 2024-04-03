// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { Network } from '@roochnetwork/rooch-sdk'

export type ClientActions = {
  addNetwork: (network: Network) => void
  switchNetwork: (network: Network) => void
  removeNetwork: (network: Network) => void
}

export type ClientStore = {
  networks: Network[]
  currentNetwork: Network
  // lastConnectedNetwork: Network | null
} & ClientActions

type ClientConfiguration = {
  storage: StateStorage
  storageKey: string
  networks: Network[]
  currentNetwork: Network
}

export function createClientStore({ storage, storageKey, currentNetwork }: ClientConfiguration) {
  return createStore<ClientStore>()(
    persist(
      (set, get) => ({
        networks: [],
        currentNetwork: currentNetwork,
        // lastConnectedNetwork: null,
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
            // lastConnectedNetwork: network,
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ networks, currentNetwork }) => ({
          networks,
          currentNetwork,
          // lastConnectedNetwork,
        }),
      },
    ),
  )
}
