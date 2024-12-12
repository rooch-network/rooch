// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'

import { Session } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../constants/index.js'
import { useCurrentSession, useRoochClient, useSessions } from './index.js'
import { useSessionStore } from './useSessionsStore.js'

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
  const sessionsKeys = useSessions()
  const removeSession = useSessionStore((state) => state.removeSession)
  const setCurrentSession = useSessionStore((state) => state.setCurrentSession)
  const currentSession = useCurrentSession()
  const client = useRoochClient()

  return useMutation({
    mutationKey: roochMutationKeys.removeSession(mutationKey),
    mutationFn: async (args) => {
      try {
        if (!currentSession) {
          return
        }

        const result = await client.removeSession({
          authKey: args.authKey,
          signer: currentSession,
        })

        if (result) {
          // clean cache
          let cacheSession = sessionsKeys.find(
            (item: Session) => item.getAuthKey() === args.authKey,
          )

          if (cacheSession) {
            removeSession(cacheSession)
            if (cacheSession.getAuthKey() === currentSession?.getAuthKey()) {
              setCurrentSession(undefined)
            }
          }
        }
      } catch (e) {
        throw e
      }
    },
    ...mutationOptions,
  })
}
