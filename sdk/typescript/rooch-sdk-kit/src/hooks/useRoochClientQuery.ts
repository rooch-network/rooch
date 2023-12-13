// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { RoochClient } from '@roochnetwork/rooch-sdk'
import type { UseQueryOptions, UseQueryResult } from '@tanstack/react-query'
import { useQuery } from '@tanstack/react-query'

import type { PartialBy } from '../types/utilityTypes'
import { useRoochClientContext } from './useRoochClient'

export type RpcMethodName = {
  [K in keyof RoochClient]: RoochClient[K] extends
    | ((input: any) => Promise<any>)
    | (() => Promise<any>)
    ? K
    : never
}[keyof RoochClient]

export type RpcMethods = {
  [K in RpcMethodName]: RoochClient[K] extends (input: infer P) => Promise<infer R>
    ? {
        name: K
        result: R
        params: P
      }
    : RoochClient[K] extends () => Promise<infer R>
    ? {
        name: K
        result: R
        params: undefined | object
      }
    : never
}

export type UseRoochClientQueryOptions<T extends keyof RpcMethods, TData> = PartialBy<
  Omit<UseQueryOptions<RpcMethods[T]['result'], Error, TData, unknown[]>, 'queryFn'>,
  'queryKey'
>

export function useRoochClientQuery<T extends keyof RpcMethods, TData = RpcMethods[T]['result']>(
  ...args: undefined extends RpcMethods[T]['params']
    ? [method: T, params?: RpcMethods[T]['params'], options?: UseRoochClientQueryOptions<T, TData>]
    : [method: T, params: RpcMethods[T]['params'], options?: UseRoochClientQueryOptions<T, TData>]
): UseQueryResult<TData, Error> {
  const [method, params, { queryKey = [], ...options } = {}] = args as [
    method: T,
    params?: RpcMethods[T]['params'],
    options?: UseRoochClientQueryOptions<T, TData>,
  ]

  const roochContext = useRoochClientContext()

  return useQuery({
    ...options,
    queryKey: [roochContext.chain, method, params, ...queryKey],
    queryFn: async () => {
      return await roochContext.client[method](params as never)
    },
  })
}
