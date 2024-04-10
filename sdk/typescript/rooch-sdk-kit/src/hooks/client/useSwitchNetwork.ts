// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Network } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochContextStore } from './index'

type UseSwitchNetworkArgs = Network

type UseSwitchNetworkResult = void

type UseSwitchNetworkMutationOptions = Omit<
  UseMutationOptions<UseSwitchNetworkResult, Error, UseSwitchNetworkArgs, unknown>,
  'mutationFn'
>

export function useSwitchNetwork({
  mutationKey,
  ...mutationOptions
}: UseSwitchNetworkMutationOptions = {}): UseMutationResult<
  UseSwitchNetworkResult,
  Error,
  UseSwitchNetworkArgs,
  unknown
> {
  const switchNetwork = useRoochContextStore((state) => state.switchNetwork)

  return useMutation({
    mutationKey: roochMutationKeys.switchNetwork(mutationKey),
    mutationFn: async (args) => {
      switchNetwork(args)
    },
    ...mutationOptions,
  })
}
