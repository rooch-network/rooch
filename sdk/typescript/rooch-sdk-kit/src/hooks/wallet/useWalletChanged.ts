// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from '../../wallet/index.js'
import { useWalletStore } from './useWalletStore.js'
import { useEffect } from 'react'
import { getWallets } from '../../wallet/wallets.js'
import { getRegisteredWallets } from '../../wallet/util.js'

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
