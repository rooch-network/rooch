// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'

import { Session } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../constants/index.js'
import { useCurrentSession, useRoochClient, useRoochSessionStore, useSession } from './index.js'

type UseRemoveSessionArgs = {
  authKey: string
}

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
  const sessionsKeys = useSession()
  const removeSession = useRoochSessionStore((state) => state.removeSession)
  const client = useRoochClient()
  const curSessionKey = useCurrentSession()

  return useMutation({
    mutationKey: roochMutationKeys.removeSession(mutationKey),
    mutationFn: async (args) => {
      try {
        if (!curSessionKey) {
          return
        }

        const result = await client.removeSession({
          authKey: curSessionKey.getAuthKey(),
          signer: curSessionKey,
        })

        if (result) {
          // clean cache
          let cacheSession = sessionsKeys.find(
            (item: Session) => item.getAuthKey() === args.authKey,
          )

          if (cacheSession) {
            removeSession(cacheSession)
          }
        }
      } catch (e) {
        throw e
      }
    },
    ...mutationOptions,
  })
}
