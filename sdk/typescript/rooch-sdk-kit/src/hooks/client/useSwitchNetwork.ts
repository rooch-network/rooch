// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Network } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochContext } from '../../hooks/useRoochContext'

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
  const { switchNetwork } = useRoochContext()

  return useMutation({
    mutationKey: roochMutationKeys.switchNetwork(mutationKey),
    mutationFn: async (args) => {
      switchNetwork(args)
    },
    ...mutationOptions,
  })
}
