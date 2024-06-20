// // Copyright (c) RoochNetwork
// // SPDX-License-Identifier: Apache-2.0
//
// import { createStore } from 'zustand'
// import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
// import { RoochClient } from '@roochnetwork/rooch-sdk'
// import { NetworkConfig } from '@/hooks'
//
// export type NetworkConfigs<T extends NetworkConfig | RoochClient = NetworkConfig | RoochClient> =
//   Record<string, T>
//
// export type ClientContextActions = {
//   addNetwork: (input: NetworkConfig) => void
//   switchNetwork: (input: string) => void
//   removeNetwork: (input: string) => void
// }
//
// export type ClientContextStoreState = {
//   networks: NetworkConfigs
//   currentNetwork: string
// } & ClientContextActions
//
// export type ClientContextStore = ReturnType<typeof createClientContextStore>
//
// type ClientContextConfiguration = {
//   storage: StateStorage
//   storageKey: string
//   networks: NetworkConfigs
//   currentNetwork: string
// }
//
// export function createClientContextStore({
//   storage,
//   storageKey,
//   networks,
//   currentNetwork,
// }: ClientContextConfiguration) {
//   return createStore<ClientContextStoreState>()(
//     persist(
//       (set, get) => ({
//         networks: networks,
//         currentNetwork: currentNetwork,
//         addNetwork(input) {
//           const cache = get().networks
//           set(() => ({
//             networks: {
//               ...cache,
//               input,
//             },
//           }))
//         },
//         removeNetwork(input) {
//           const cache = get().networks
//           const { [input]: _, ...remainingNetworks } = cache
//           set(() => ({
//             networks: remainingNetworks,
//           }))
//         },
//         switchNetwork(network) {
//           set(() => ({
//             currentNetwork: network,
//           }))
//         },
//       }),
//       {
//         name: storageKey,
//         storage: createJSONStorage(() => storage),
//         partialize: ({ networks, currentNetwork }) => ({
//           networks,
//           currentNetwork,
//         }),
//       },
//     ),
//   )
// }
