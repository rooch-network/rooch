// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useQuery } from '@tanstack/react-query'
import { useLayoutEffect, useState } from 'react'

import {
  useWalletStore,
  useConnectWallet,
  useWallets,
  useCurrentWallet,
  useCurrentAddress,
} from './index.js'

export function useAutoConnectWallet(): 'disabled' | 'idle' | 'attempted' {
  const { mutateAsync: connectWallet } = useConnectWallet()
  const autoConnectEnabled = useWalletStore((state) => state.autoConnectEnabled)
  const lastConnectedWalletName = useWalletStore((state) => state.lastConnectedWalletName)
  const lastConnectedAddress = useWalletStore((state) => state.lastConnectedAddress)
  const { isConnected } = useCurrentWallet()
  const wallets = useWallets()
  const [clientOnly, setClientOnly] = useState(false)
  const currentAddress = useCurrentAddress()

  useLayoutEffect(() => {
    setClientOnly(true)
  }, [])

  const { data, isError } = useQuery({
    queryKey: [
      '@rooch/sdk-kit',
      'autoconnect',
      {
        isConnected,
        autoConnectEnabled,
        lastConnectedWalletName,
        lastConnectedAddress,
      },
    ],
    queryFn: async () => {
      if (!autoConnectEnabled) {
        return 'disabled'
      }

      if (!lastConnectedWalletName || !lastConnectedAddress || isConnected) {
        return 'attempted'
      }

      let wallet = wallets.find((wallet) => wallet.getName() === lastConnectedWalletName)

      if (wallet) {
        await connectWallet({ wallet })
        if (wallet.getChain() !== 'bitcoin' && currentAddress?.toStr() !== lastConnectedAddress) {
          wallet.switchAccount(lastConnectedAddress)
        }
      }

      return 'attempted'
    },
    enabled: autoConnectEnabled,
    persister: undefined,
    gcTime: 0,
    staleTime: 0,
    networkMode: 'always',
    retry: (failureCount) => {
      // Retry only if there is a wallet to connect and we haven't exceeded 3 attempts
      if (
        wallets.find((wallet) => wallet.getName() === lastConnectedWalletName) &&
        failureCount < 3
      ) {
        return true
      }
      return false
    },
    retryOnMount: false,
    refetchInterval: 1000,
    refetchIntervalInBackground: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
    refetchOnWindowFocus: false,
  })

  if (!autoConnectEnabled) {
    return 'disabled'
  }

  // We always initialize with "idle" so that in SSR environments, we guarantee that the initial render states always agree:
  if (!clientOnly) {
    return 'idle'
  }

  if (!lastConnectedWalletName) {
    return 'attempted'
  }

  return isError ? 'attempted' : data ?? 'idle'
}
