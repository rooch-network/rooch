// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Network } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochContext } from '../../hooks/useRoochContext'

type UseRemoveNetworkArgs = Network

type UseRemoveNetworkResult = void

type UseRemoveNetworkMutationOptions = Omit<
  UseMutationOptions<UseRemoveNetworkResult, Error, UseRemoveNetworkArgs, unknown>,
  'mutationFn'
>

export function useRemoveNetwork({
  mutationKey,
  ...mutationOptions
}: UseRemoveNetworkMutationOptions = {}): UseMutationResult<
  UseRemoveNetworkResult,
  Error,
  UseRemoveNetworkArgs,
  unknown
> {
  const { removeNetwork } = useRoochContext()

  return useMutation({
    mutationKey: roochMutationKeys.removeNetwork(mutationKey),
    mutationFn: async (args) => {
      removeNetwork(args)
    },
    ...mutationOptions,
  })
}
