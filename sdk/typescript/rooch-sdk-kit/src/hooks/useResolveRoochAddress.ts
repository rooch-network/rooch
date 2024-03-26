// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseQueryOptions, UseQueryResult } from '@tanstack/react-query'

import { useRoochClientQuery } from './useRoochClientQuery'
import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'

export function useResolveRoochAddress(
  address: string,
  multiChainID: RoochMultiChainID,
  options?: Omit<UseQueryOptions<string, Error, string | null, unknown[]>, 'queryFn' | 'queryKey'>,
): UseQueryResult<string | null, Error> {
  return useRoochClientQuery(
    'resoleRoochAddress',
    {
      address,
      multiChainID: multiChainID,
    },
    {
      ...options,
      refetchOnWindowFocus: false,
      retry: false,
      enabled: !!address && options?.enabled !== false,
    },
  )
}
