// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from '../../wellet/index.js'
import { useWalletStore } from './useWalletStore.js'
import { useEffect } from 'react'
import { getWallets } from '../../wellet/wallets.js'
import { getRegisteredWallets } from '../../wellet/util.js'

/**
 * Retrieves all wallets
 */
export function useWalletChanged(
  preferredWallets: string[],
  walletFilter?: (wallet: Wallet) => boolean,
) {
  const updateWallets = useWalletStore((state) => state.updateWallets)

  useEffect(() => {
    const api = getWallets()
    updateWallets(getRegisteredWallets(preferredWallets, walletFilter))

    const unsubscribeFromRegister = api.on('register', () => {
      updateWallets(getRegisteredWallets(preferredWallets, walletFilter))
    })

    return () => {
      unsubscribeFromRegister()
    }
  }, [preferredWallets, updateWallets, walletFilter])
}
