// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Network } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochContextStore } from './index'

type UseAddNetworkArgs = Network

type UseAddNetworkResult = void

type UseAddNetworkMutationOptions = Omit<
  UseMutationOptions<UseAddNetworkResult, Error, UseAddNetworkArgs, unknown>,
  'mutationFn'
>

export function useAddNetwork({
  mutationKey,
  ...mutationOptions
}: UseAddNetworkMutationOptions = {}): UseMutationResult<
  UseAddNetworkResult,
  Error,
  UseAddNetworkArgs,
  unknown
> {
  const addNetwork = useRoochContextStore((state) => state.addNetwork)

  return useMutation({
    mutationKey: roochMutationKeys.addNetwork(mutationKey),
    mutationFn: async (args) => {
      addNetwork(args)
    },
    ...mutationOptions,
  })
}
