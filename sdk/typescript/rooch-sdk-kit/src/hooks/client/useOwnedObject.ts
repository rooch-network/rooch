// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// import { useInfiniteQuery } from '@tanstack/react-query'
// import { IndexerStateID, ObjectStateFilterView, ObjectStateView } from '@roochnetwork/rooch-sdk'
// import { useRoochClient } from './useRoochClient'

// const MAX_LIMIT = 10

// Rooch type not currently supported

// export function useOwnedObject(address: string, filter?: ObjectStateFilterView, limit = MAX_LIMIT) {
//   const client = useRoochClient()
//   return useInfiniteQuery<ObjectStateView>({
//     initialPageParam: null,
//     queryKey: ['get-owned-objects', address, filter, limit],
//     queryFn: ({ pageParam }) =>
//       client.queryGlobalStates({
//         filter: filter || { owner: address },
//         limit: limit,
//         cursor: (pageParam as IndexerStateID) || null,
//       }),
//     staleTime: 10 * 1000,
//     enabled: !!address,
//     getNextPageParam: ({ has_next_page, next_cursor }) => (has_next_page ? next_cursor : null),
//   })
// }
