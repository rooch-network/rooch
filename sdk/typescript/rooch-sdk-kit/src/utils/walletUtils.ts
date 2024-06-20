// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain, SupportChains } from '@/feature'
import { UniSatWallet, Wallet } from '@/wellet'

export async function checkWallets(filter?: SupportChain) {
  const wallets: Wallet[] = []
  SupportChains.filter((v) => !filter || filter === v).forEach((w) => {
    switch (w) {
      case SupportChain.BITCOIN:
        wallets.push(new UniSatWallet(false))
    }
  })

  return await Promise.all(
    wallets.map(async (w) => {
      if (await w.checkInstalled()) {
        return new UniSatWallet(true)
      }
      return w
    }),
  )
}
