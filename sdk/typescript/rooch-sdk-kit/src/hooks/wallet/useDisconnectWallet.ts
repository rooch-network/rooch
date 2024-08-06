// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useMutation, UseMutationOptions } from '@tanstack/react-query'
import { useWalletStore } from './useWalletStore.js'
import { walletMutationKeys } from '../../constants/index.js'
import { useCurrentWallet } from './useCurrentWallet.js'
import { WalletNotConnectedError } from '../../error/walletErrors.js'

type UseDisconnectWalletError = WalletNotConnectedError | Error

type UseDisconnectWalletMutationOptions = Omit<
  UseMutationOptions<void, UseDisconnectWalletError, void, unknown>,
  'mutationFn'
>

/**
 * Mutation hook for disconnecting from an active wallet connection, if currently connected.
 */
export function useDisconnectWallet({
  mutationKey,
  ...mutationOptions
}: UseDisconnectWalletMutationOptions = {}) {
  const { wallet } = useCurrentWallet()

  const setWalletDisconnected = useWalletStore((state) => state.setWalletDisconnected)

  return useMutation({
    mutationKey: walletMutationKeys.disconnectWallet(mutationKey),
    mutationFn: async () => {
      if (!wallet) {
        throw new WalletNotConnectedError('No wallet is connected.')
      }
      setWalletDisconnected()
    },
    ...mutationOptions,
  })
}
