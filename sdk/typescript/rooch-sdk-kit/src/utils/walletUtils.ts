// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain, SupportChains } from '../feature'
import { BaseWallet, Metamask, UniSatWallet } from '../types'

export async function getInstalledWallets(filter?: SupportChain) {
  const wallets: BaseWallet[] = []
  SupportChains.filter((v) => !filter || filter === v).forEach((w) => {
    switch (w) {
      case SupportChain.ETH:
        wallets.push(new Metamask())
        break
      case SupportChain.BITCOIN:
        wallets.push(new UniSatWallet())
        break
      case SupportChain.Rooch:
        wallets.push(new Metamask(), new UniSatWallet())
        break
    }
  })

  const installWallets = await Promise.all(
    wallets.map(async (w) => {
      if (await w.checkInstalled()) {
        w.installed = true
        return w
      }
      return undefined
    }),
  )

  return installWallets.filter((w): w is BaseWallet => w !== undefined)
}
