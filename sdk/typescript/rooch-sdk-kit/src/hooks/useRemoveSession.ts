// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { roochMutationKeys } from '../constants/roochMutationKeys'
import { useCurrentSession, useRoochClient, useRoochSessionStore, useSession } from './index'
import { addressToSeqNumber } from '@roochnetwork/rooch-sdk'

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

        const result = await client.executeTransaction({
          funcId: '0x3::session_key::remove_session_key_entry',
          args: [
            {
              type: { Vector: 'U8' },
              value: addressToSeqNumber(args.authKey),
            },
          ],
          tyArgs: [],
          address: curSessionKey.getAddress(),
          authorizer: curSessionKey.getAuthorizer(),
        })

        if (result.execution_info.status.type === 'executed') {
          // clean cache
          let cacheSession = sessionsKeys.find((item) => item.getAuthKey() === args.authKey)

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
