// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { RoochSessionAccount } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../constants/roochMutationKeys'
import { useRoochSessionStore } from './index'

type UseRemoveSessionArgs = RoochSessionAccount

type UseRemoveSessionResult = void

type UseRemoveSessionMutationOptions = Omit<
  UseMutationOptions<UseRemoveSessionResult, Error, UseRemoveSessionArgs, unknown>,
  'mutationFn'
>

export function useRemoveSession({
  mutationKey,
  ...mutationOptions
}: UseRemoveSessionMutationOptions = {}): UseMutationResult<
  UseRemoveSessionResult,
  Error,
  UseRemoveSessionArgs,
  unknown
> {
  const removeSession = useRoochSessionStore((state) => state.removeSession)

  return useMutation({
    mutationKey: roochMutationKeys.removeSession(mutationKey),
    mutationFn: async (args) => {
      try {
        // args.removeSessionKey()
      } catch (e) {
        throw e
      }

      removeSession(args)
    },
    ...mutationOptions,
  })
}
