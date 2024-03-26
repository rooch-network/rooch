// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'

import { useWalletStore } from './useWalletStore'
import { BaseWallet, WalletAccount } from '../../types'
import { walletMutationKeys } from '../../constants/walletMutationKeys'
import { useRoochClient } from '../../hooks/useRoochClient'
import { chain2MultiChainID } from '../../utils/chain2MultiChainID'

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
  // const currentAccount = useWalletStore((state) => state.currentAccount)
  const chain = useWalletStore((state) => state.currentChain)
  const client = useRoochClient()

  return useMutation({
    mutationKey: walletMutationKeys.connectWallet(mutationKey),
    mutationFn: async ({ wallet }) => {
      try {
        setConnectionStatus('connecting')

        const connectAccounts = await wallet.connect()
        const selectedAccount = connectAccounts[0]

        // use cache date
        // if (selectedAccount.address === currentAccount?.address) {
        //   setWalletConnected(connectAccounts, currentAccount)
        //   return connectAccounts
        // }

        let selectedAccountRoochAddress = await client.resoleRoochAddress({
          address: selectedAccount.address,
          multiChainID: chain2MultiChainID(chain),
        })

        setWalletConnected(
          wallet,
          connectAccounts,
          new WalletAccount(
            selectedAccount.address,
            selectedAccountRoochAddress,
            selectedAccount.walletType,
            selectedAccount.publicKey,
            selectedAccount.compressedPublicKey,
          ),
        )

        return connectAccounts
      } catch (error) {
        setConnectionStatus('disconnected')
        throw error
      }
    },
    ...mutationOptions,
  })
}
