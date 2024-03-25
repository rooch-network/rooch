// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useWalletStore } from './useWalletStore'

/**
 * Retrieves the wallet that is currently connected to the dApp, if one exists.
 */
export function useCurrentWallet() {
  const currentWallet = useWalletStore((state) => state.currentWallet)
  const connectionStatus = useWalletStore((state) => state.connectionStatus)
  switch (connectionStatus) {
    case 'connecting':
      return {
        connectionStatus,
        currentWallet: currentWallet,
        isDisconnected: false,
        isConnecting: true,
        isConnected: false,
      } as const
    case 'disconnected':
      return {
        connectionStatus,
        currentWallet: currentWallet,
        isDisconnected: true,
        isConnecting: false,
        isConnected: false,
      } as const
    case 'connected': {
      return {
        connectionStatus,
        currentWallet: currentWallet!,
        isDisconnected: false,
        isConnecting: false,
        isConnected: true,
      } as const
    }
  }
}
