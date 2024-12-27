// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Session } from '@roochnetwork/rooch-sdk'
import type { CreateSessionArgs, Signer } from '@roochnetwork/rooch-sdk'

import { useRoochClient } from './client/index.js'
import { useCurrentWallet } from './wallet/useCurrentWallet.js'
import { walletMutationKeys } from '../constants/index.js'
import { WalletNotConnectedError } from '../error/walletErrors.js'
import { useSessionStore } from './useSessionsStore.js'
import { useTriggerError } from '../provider/globalProvider.js'

type UseCreateSessionKeyArgs = {
  signer?: Signer
} & CreateSessionArgs

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
  const { wallet } = useCurrentWallet()
  const setCurrentSession = useSessionStore((state) => state.setCurrentSession)
  const triggerError = useTriggerError()
  return useMutation({
    mutationKey: walletMutationKeys.createSessionKey(mutationKey),
    mutationFn: async (args) => {
      const signer = args.signer || wallet
      if (!signer) {
        throw new WalletNotConnectedError('No wallet is connected.')
      }
      try {
        const sessionAccount = await client.createSession({
          signer: signer,
          sessionArgs: args,
        })

        setCurrentSession(sessionAccount)

        return sessionAccount
      } catch (error: any) {
        if ('code' in error && 'message' in error) {
          triggerError(error)
        }
        throw error
      }
    },
    ...mutationOptions,
  })
}
