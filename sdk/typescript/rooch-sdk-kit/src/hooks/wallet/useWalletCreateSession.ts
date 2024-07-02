// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Session } from '@roochnetwork/rooch-sdk'
import type { CreateSessionArgs } from '@roochnetwork/rooch-sdk'

import { useRoochClient } from '../client/index.js'
import { useCurrentWallet } from './useCurrentWallet.js'
import { walletMutationKeys } from '../../constants/index.js'
import { WalletNotConnectedError } from '../../error/walletErrors.js'
import { useSessionStore } from '../useSessionsStore.js'

type UseCreateSessionKeyArgs = CreateSessionArgs

type UseCreateSessionKeyError = WalletNotConnectedError | Error

type UseCreateSessionKeyResult = Session | null

type UseCreateSessionKeyMutationOptions = Omit<
  UseMutationOptions<
    UseCreateSessionKeyResult,
    UseCreateSessionKeyError,
    UseCreateSessionKeyArgs,
    unknown
  >,
  'mutationFn'
>

export function useCreateSessionKey({
  mutationKey,
  ...mutationOptions
}: UseCreateSessionKeyMutationOptions = {}): UseMutationResult<
  UseCreateSessionKeyResult,
  UseCreateSessionKeyError,
  UseCreateSessionKeyArgs,
  unknown
> {
  const client = useRoochClient()
  const currentWallet = useCurrentWallet()
  const setCurrentSession = useSessionStore((state) => state.setCurrentSession)

  return useMutation({
    mutationKey: walletMutationKeys.createSessionKey(mutationKey),
    mutationFn: async (args) => {
      if (!currentWallet.isConnected) {
        throw new WalletNotConnectedError('No wallet is connected.')
      }

      const sessionAccount = await client.createSession({
        signer: currentWallet.wallet!,
        sessionArgs: args,
      })

      setCurrentSession(sessionAccount)

      return sessionAccount
    },
    ...mutationOptions,
  })
}
