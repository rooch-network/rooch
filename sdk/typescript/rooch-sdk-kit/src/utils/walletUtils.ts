// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain } from '../feature/index.js'
import { UniSatWallet, Wallet, OkxWallet } from '../wellet/index.js'

export async function checkWallets(filter?: SupportChain) {
  const wallets: Wallet[] = [new UniSatWallet(), new OkxWallet()].filter(
    (wallet) => wallet.getChain() === filter || !filter,
  )

  return await Promise.all(wallets.filter(async (w) => await w.checkInstalled()))
}
