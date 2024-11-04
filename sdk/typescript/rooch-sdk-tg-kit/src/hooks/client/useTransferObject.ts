// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Signer, TypeArgs } from '@roochnetwork/rooch-sdk'
import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'

import { useRoochClient } from './useRoochClient.js'
import { roochMutationKeys } from '../../constants/index.js'

type UseTransferObjectArgs = {
  signer: Signer
  recipient: string
  objectId: string
  objectType: TypeArgs
}

type UseTransferObjectResult = void

type UseSwitchNetworkMutationOptions = Omit<
  UseMutationOptions<UseTransferObjectResult, Error, UseTransferObjectArgs, unknown>,
  'mutationFn'
>

export function useTransferObject({
  mutationKey,
  ...mutationOptions
}: UseSwitchNetworkMutationOptions = {}): UseMutationResult<
  UseTransferObjectResult,
  Error,
  UseTransferObjectArgs,
  unknown
> {
  const client = useRoochClient()

  return useMutation({
    mutationKey: roochMutationKeys.transferObject(mutationKey),
    mutationFn: async (args) => {
      const result = await client.transferObject(args)

      if (result.execution_info.status.type !== 'executed') {
        Error('transfer failed' + result.execution_info.status.type)
      }
    },
    ...mutationOptions,
  })
}
