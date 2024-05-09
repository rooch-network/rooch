// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'

import { useWalletStore } from './useWalletStore'
import { BaseWallet, WalletAccount } from '../../types'
import { walletMutationKeys } from '../../constants/walletMutationKeys'
import { Buffer } from 'buffer'

type ConnectWalletArgs = {
  wallet: BaseWallet
}
type ConnectWalletResult = WalletAccount[]

type UseConnectWalletMutationOptions = Omit<
  UseMutationOptions<ConnectWalletResult, Error, ConnectWalletArgs, unknown>,
  'mutationFn'
>

/**
 * Mutation hook for establishing a connection to a specific wallet.
 *
 */
export function useConnectWallet({
  mutationKey,
  ...mutationOptions
}: UseConnectWalletMutationOptions = {}): UseMutationResult<
  ConnectWalletResult,
  Error,
  ConnectWalletArgs,
  unknown
> {
  const setWalletConnected = useWalletStore((state) => state.setWalletConnected)
  const setConnectionStatus = useWalletStore((state) => state.setConnectionStatus)

  return useMutation({
    mutationKey: walletMutationKeys.connectWallet(mutationKey),
    mutationFn: async ({ wallet }) => {
      try {
        setConnectionStatus('connecting')

        const connectAccounts = await wallet.connect()
        const selectedAccount = connectAccounts[0]
        await selectedAccount.resoleRoochAddress()

        setWalletConnected(wallet, connectAccounts, selectedAccount)

        console.log(selectedAccount)
        console.log(Buffer.from(selectedAccount.address).toString('hex'))

        return connectAccounts
      } catch (error) {
        setConnectionStatus('disconnected')
        throw error
      }
    },
    ...mutationOptions,
  })
}
