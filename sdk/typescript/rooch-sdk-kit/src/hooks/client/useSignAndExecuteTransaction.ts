// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { UseMutationOptions, UseMutationResult, useMutation } from '@tanstack/react-query'

import { Signer, Transaction, ExecuteTransactionResponseView } from '@roochnetwork/rooch-sdk'

import { useRoochClient } from './useRoochClient.js'
import { roochMutationKeys } from '../../constants/index.js'
import { useCurrentSession } from '../useCurrentSession.js'

type UseSignAndExecuteTransactionArgs = {
  transaction: Transaction
  signer?: Signer
}

type UsesignAndExecuteTransactionResult = ExecuteTransactionResponseView

type UsesignAndExecuteTransactionOptions = Omit<
  UseMutationOptions<
    UsesignAndExecuteTransactionResult,
    Error,
    UseSignAndExecuteTransactionArgs,
    unknown
  >,
  'mutationFn'
>

export function useSignAndExecuteTransaction({
  mutationKey,
  ...mutationOptions
}: UsesignAndExecuteTransactionOptions = {}): UseMutationResult<
  UsesignAndExecuteTransactionResult,
  Error,
  UseSignAndExecuteTransactionArgs,
  unknown
> {
  const client = useRoochClient()
  const session = useCurrentSession()

  return useMutation({
    mutationKey: roochMutationKeys.signAndExecuteTransaction(mutationKey),
    mutationFn: async (args) => {
      if (!session) {
        throw Error('Create a session first')
      }

      const result = await client.signAndExecuteTransaction({
        transaction: args.transaction,
        signer: args.signer || session,
      })

      if (result.execution_info.status.type !== 'executed' && result.execution_info.status) {
        Error('transfer failed' + result.execution_info.status.type)
      }

      return result
    },
    ...mutationOptions,
  })
}
