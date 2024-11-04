// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { address, TypeArgs, Signer } from '@roochnetwork/rooch-sdk'

import { roochMutationKeys } from '../../constants/index.js'
import { useRoochClient } from './index.js'
import { useCurrentSession } from '../useCurrentSession.js'

type UseTransferCoinArgs = {
  signer?: Signer
  recipient: address
  amount: number | bigint
  coinType: TypeArgs
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
  const curSession = useCurrentSession()

  return useMutation({
    mutationKey: roochMutationKeys.transferCoin(mutationKey),
    mutationFn: async (args) => {
      const signer = args.signer || curSession

      if (signer === null) {
        throw Error('')
      }

      const result = await client.transfer({
        ...args,
        signer: args.signer || curSession!,
      })

      if (result.execution_info.status.type !== 'executed') {
        Error('transfer failed' + result.execution_info.status.type)
      }
    },
    ...mutationOptions,
  })
}
