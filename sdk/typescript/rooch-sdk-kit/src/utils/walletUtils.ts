// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain, SupportChains } from '../feature'
import { Metamask, UniSatWallet } from '../types/wellet'
import { BaseWallet } from '../types/wellet/baseWallet'

export async function getInstalledWallets(filter?: SupportChain) {
  const wallets = SupportChains.filter((v) => {
    if (filter) {
      return filter === v
    } else {
      return false
    }
  }).map((w) => {
    let wallet: BaseWallet
    switch (w) {
      case SupportChain.ETH:
        wallet = new Metamask()
        break
      case SupportChain.BITCOIN:
        wallet = new UniSatWallet()
        break
    }

    return wallet
  })

  return wallets.filter(async (w) => await w.checkInstalled())
}
