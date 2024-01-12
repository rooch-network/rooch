// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportWallet, SupportWallets } from '../feature'
import { Metamask, UniSatWallet } from '../types/wellet'
import { BaseWallet } from '../types/wellet/baseWallet'

export async function getInstalledWallets(filter?: SupportWallet) {
  const wallets = SupportWallets.filter((v) => {
    if (filter) {
      return filter === v
    } else {
      return false
    }
  }).map((w) => {
    let wallet: BaseWallet
    switch (w) {
      case SupportWallet.ETH:
        wallet = new Metamask()
        break
      case SupportWallet.BITCOIN:
        wallet = new UniSatWallet()
        break
      // case SupportWallet.Okx:
      //   wallet = new OkxWallet()
      //   break
    }

    return wallet
  })

  return wallets.filter(async (w) => await w.checkInstalled())
}
