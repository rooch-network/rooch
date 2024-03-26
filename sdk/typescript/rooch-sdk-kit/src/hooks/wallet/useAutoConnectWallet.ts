// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useQuery } from '@tanstack/react-query'
import { useLayoutEffect, useState } from 'react'

import { useWalletStore } from './useWalletStore'
import { useConnectWallet } from './useConnectWallet'
import { useSupportWallets } from '../../hooks/wallet/useWallets'
import { useCurrentWallet } from '../../hooks/wallet/useCurrentWallet'

export function useAutoConnectWallet(): 'disabled' | 'idle' | 'attempted' {
  const { mutateAsync: connectWallet } = useConnectWallet()
  const autoConnectEnabled = useWalletStore((state) => state.autoConnectEnabled)
  const lastConnectedWalletName = useWalletStore((state) => state.lastConnectedWalletName)
  const lastConnectedAccountAddress = useWalletStore((state) => state.lastConnectedAccountAddress)
  const { isConnected } = useCurrentWallet()
  const wallets = useSupportWallets()
  const [clientOnly, setClientOnly] = useState(false)

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
        lastConnectedAccountAddress,
      },
    ],
    queryFn: async () => {
      if (!autoConnectEnabled) {
        return 'disabled'
      }

      if (!lastConnectedWalletName || !lastConnectedAccountAddress || isConnected) {
        return 'attempted'
      }

      let wallet = wallets.find((wallet) => wallet.name === lastConnectedWalletName)

      if (wallet) {
        await connectWallet({ wallet })
      }

      // TODO: switch lastConnectedAccount

      return 'attempted'
    },
    enabled: autoConnectEnabled,
    persister: undefined,
    gcTime: 0,
    staleTime: 0,
    networkMode: 'always',
    retry: false,
    retryOnMount: false,
    refetchInterval: false,
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
