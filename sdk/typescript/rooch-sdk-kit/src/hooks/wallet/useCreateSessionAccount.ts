// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { Account, IAccount, RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { useWalletStore } from './useWalletStore'
import { useRoochClient } from '../useRoochClient'
import { WalletAuth } from '../../auth/walletAuth'
import { useCurrentWallet } from './useCurrentWallet'
import { useResolveRoochAddress } from '../useResolveRoochAddress'
import { walletMutationKeys } from '../../constants/walletMutationKeys'

interface UseCreateSessionKeyArgs {
  scope?: string[]
  maxInactiveInterval?: number
}

type UseCreateSessionKeyResult = IAccount | null

type UseCreateSessionKeyMutationOptions = Omit<
  UseMutationOptions<UseCreateSessionKeyResult, Error, UseCreateSessionKeyArgs, unknown>,
  'mutationFn'
>

export const defaultScope = [
  '0x1::*::*',
  '0x3::*::*',
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::*::*',
]

export function useCreateSessionKey({
  mutationKey,
  ...mutationOptions
}: UseCreateSessionKeyMutationOptions = {}): UseMutationResult<
  UseCreateSessionKeyResult,
  Error,
  UseCreateSessionKeyArgs,
  unknown
> {
  const rooch = useRoochClient()
  const currentWallet = useCurrentWallet()
  const currentAccount = useWalletStore((state) => state.currentAccount)
  let roochAddress = useResolveRoochAddress(
    currentAccount?.address ?? '',
    RoochMultiChainID.Bitcoin,
  )
  // TODO: save session with account & scope
  const sessionKey = useWalletStore((state) => state.sessionAccount)
  const setSessionAccount = useWalletStore((state) => state.setSessionAccount)

  return useMutation({
    mutationKey: walletMutationKeys.createSessionKey(mutationKey),
    mutationFn: async (args) => {
      if (sessionKey) {
        // TODO: Recover from cache
      }

      // if (!currentWallet) {
      //   throw new WalletNotConnectedError('No wallet is connected.');
      // }

      // TODO: check currentWallet === signerAccount
      // throw new WalletNoAccountSelectedError(
      //   'No wallet account is selected to sign the personal message with.',
      // );

      let scope = args.scope ?? defaultScope
      let maxInactiveInterval = args.maxInactiveInterval ?? 1200

      let acc = new Account(
        rooch,
        roochAddress.data!,
        new WalletAuth(
          currentWallet.currentWallet,
          `Welcome to ${window.location.hostname}\nYou will authorize session:\n${
            'Scope:\n' +
            scope
              .filter((v) => !v.startsWith('0x1') && !v.startsWith('0x3'))
              .map((v) => {
                console.log(v)
                return v
              }) +
            '\nTimeOut:' +
            maxInactiveInterval
          }`,
        ),
      )

      // TODO: Standardize error and throw it at the developer
      try {
        let sessionKey = await acc.createSessionAccount(scope, maxInactiveInterval)

        setSessionAccount(sessionKey)

        return sessionKey
      } catch (e: any) {
        console.log(e.toString())
      }

      return null
    },
    ...mutationOptions,
  })
}
