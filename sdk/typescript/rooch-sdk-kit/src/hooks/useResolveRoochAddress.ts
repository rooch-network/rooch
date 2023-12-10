// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseQueryOptions, UseQueryResult } from '@tanstack/react-query'

import { useRoochClientQuery } from './useRoochClientQuery'

export function useResolveRoochAddress(
  address: string,
  options?: Omit<UseQueryOptions<string, Error, string | null, unknown[]>, 'queryFn' | 'queryKey'>,
): UseQueryResult<string | null, Error> {
  return useRoochClientQuery('resoleRoochAddress', address, {
    ...options,
    refetchOnWindowFocus: false,
    retry: false,
    enabled: !!address && options?.enabled !== false,
  })
}
