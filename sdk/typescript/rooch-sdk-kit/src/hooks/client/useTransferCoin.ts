// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { address, ModuleArgs, Signer } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '@/constants'
import { useRoochClient } from './index'

type UseTransferCoinArgs = {
  signer: Signer
  recipient: address
  amount: number | bigint
  coinType: ModuleArgs
}

type UseTransferCoinResult = void

type UseSwitchNetworkMutationOptions = Omit<
  UseMutationOptions<UseTransferCoinResult, Error, UseTransferCoinArgs, unknown>,
  'mutationFn'
>

export function useTransferCoin({
  mutationKey,
  ...mutationOptions
}: UseSwitchNetworkMutationOptions = {}): UseMutationResult<
  UseTransferCoinResult,
  Error,
  UseTransferCoinArgs,
  unknown
> {
  const client = useRoochClient()

  return useMutation({
    mutationKey: roochMutationKeys.transferCoin(mutationKey),
    mutationFn: async (args) => {
      const result = await client.transfer({
        ...args,
      })

      if (result.execution_info.status.type !== 'executed') {
        Error('transfer failed' + result.execution_info.status.type)
      }
    },
    ...mutationOptions,
  })
}
