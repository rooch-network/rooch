// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { ChainInfo } from '@roochnetwork/rooch-sdk'

import { useWalletStore } from './useWalletStore'
import { useCurrentWallet } from './useCurrentWallet'
import { WalletAccount } from '../../types/WalletAccount'
import { walletMutationKeys } from '../../constants/walletMutationKeys'

type ConnectWalletResult = WalletAccount[]

type UseConnectWalletMutationOptions = Omit<
  UseMutationOptions<ConnectWalletResult, Error, ChainInfo, unknown>,
  'mutationFn'
>

/**
 * Mutation hook for establishing a connection to a specific wallet.
 */
export function useConnectWallet({
  mutationKey,
  ...mutationOptions
}: UseConnectWalletMutationOptions = {}): UseMutationResult<
  ConnectWalletResult,
  Error,
  ChainInfo,
  unknown
> {
  const setWalletConnected = useWalletStore((state) => state.setWalletConnected)
  const setConnectionStatus = useWalletStore((state) => state.setConnectionStatus)
  const { currentWallet } = useCurrentWallet()

  return useMutation({
    mutationKey: walletMutationKeys.connectWallet(mutationKey),
    mutationFn: async ({ ...connectArgs }) => {
      try {
        setConnectionStatus('connecting')

        const accounts = await currentWallet.connect(connectArgs)

        setWalletConnected(accounts, accounts[0])

        return accounts
      } catch (error) {
        setConnectionStatus('disconnected')
        throw error
      }
    },
    ...mutationOptions,
  })
}
