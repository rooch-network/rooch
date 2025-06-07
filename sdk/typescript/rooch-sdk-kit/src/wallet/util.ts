// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from './wallet.js'
import { getWallets } from './wallets.js'

export function getRegisteredWallets(
  preferredWallets: string[],
  walletFilter?: (wallet: Wallet) => boolean,
): Wallet[] {
  const walletsApi = getWallets()
  const wallets = walletsApi.get()

  const Wallets = wallets.filter((wallet) => !walletFilter || walletFilter(wallet))

  return [
    // Preferred wallets, in order:
    ...preferredWallets.map((name) => Wallets.find((wallet) => wallet.getName() === name)),
    // Wallets in default order:
    ...Wallets.filter((wallet) => !preferredWallets.includes(wallet.getName())),
  ].filter((wallet): wallet is Wallet => wallet !== undefined)
}
