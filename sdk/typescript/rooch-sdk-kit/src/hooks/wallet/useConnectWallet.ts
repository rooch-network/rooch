// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult, MutationKey } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { useWalletStore } from './useWalletStore'
import { ChainInfo } from '@roochnetwork/rooch-sdk'
import detectEthereumProvider from '@metamask/detect-provider'
import { useCurrentWallet } from './useCurrentWallet'
import { WalletAccount } from '../../types/WalletAccount'

function formMutationKeyFn(baseEntity: string) {
  return function mutationKeyFn(additionalKeys: MutationKey = []) {
    return [{ ...walletMutationKeys.all, baseEntity }, ...additionalKeys]
  }
}

const walletMutationKeys = {
  all: { baseScope: 'wallet' },
  connectWallet: formMutationKeyFn('connect-wallet'),
  autoConnectWallet: formMutationKeyFn('auto-connect-wallet'),
  disconnectWallet: formMutationKeyFn('disconnect-wallet'),
  switchAccount: formMutationKeyFn('switch-account'),
}

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

        console.log(connectArgs.rpcUrls)
        const hashInstallMetamask = await detectEthereumProvider({ silent: true })

        if (!hashInstallMetamask) {
          console.error('rooch sdk currently, only metamask is supported')
          setConnectionStatus('disconnected')
          return []
        }

        const chainId = (await window.ethereum?.request({ method: 'eth_chainId' })) as string

        if (chainId !== connectArgs.chainId) {
          try {
            await currentWallet.switchChain({ ...connectArgs })
          } catch (e: any) {
            console.log('connect error', e.toString())
            return []
          }
        }

        const accounts: string[] = await window.ethereum
          ?.request<string[]>({
            method: 'eth_requestAccounts',
          })
          .then((accounts: any) => {
            return accounts
          })

        const walletAccount = accounts.map((v) => new WalletAccount(v))

        setWalletConnected(walletAccount, walletAccount[0])

        return walletAccount
      } catch (error) {
        setConnectionStatus('disconnected')
        throw error
      }
    },
    ...mutationOptions,
  })
}
