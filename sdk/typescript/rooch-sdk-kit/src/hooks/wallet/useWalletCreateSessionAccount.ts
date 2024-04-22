// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { RoochSessionAccount } from '@roochnetwork/rooch-sdk'
import { useRoochClient, useCurrentAccount } from '../index'
import { useCurrentWallet } from './useCurrentWallet'
import { walletMutationKeys } from '../../constants/walletMutationKeys'
import { WalletNotConnectedError } from '../../error/walletErrors'
import { WalletRoochSessionAccount } from '../../types/WalletRoochSessionAccount'
import { useRoochSessionStore } from '../index'

interface UseCreateSessionKeyArgs {
  appName: string
  appUrl: string
  scopes: string[]
  maxInactiveInterval?: number
}

type UseCreateSessionKeyError = WalletNotConnectedError | Error

type UseCreateSessionKeyResult = RoochSessionAccount | null

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
  const setSessionAccount = useRoochSessionStore((state) => state.setCurrentSession)
  const currentAccount = useCurrentAccount()

  return useMutation({
    mutationKey: walletMutationKeys.createSessionKey(mutationKey),
    mutationFn: async (args) => {
      if (!currentWallet.isConnected) {
        throw new WalletNotConnectedError('No wallet is connected.')
      }

      let scopes = args.scopes
      let maxInactiveInterval = args.maxInactiveInterval ?? 1200

      try {
        const sessionAccount = await WalletRoochSessionAccount.CREATE(
          client,
          currentAccount!,
          args.appName,
          args.appUrl,
          scopes,
          maxInactiveInterval,
        )

        setSessionAccount(sessionAccount)

        return sessionAccount
      } catch (e: any) {
        console.log(e.toString())
      }

      return null
    },
    ...mutationOptions,
  })
}
