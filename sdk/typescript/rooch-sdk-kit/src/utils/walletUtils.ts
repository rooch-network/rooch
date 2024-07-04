// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain, SupportChains } from '../feature/index.js'
import { UniSatWallet, Wallet } from '../wellet/index.js'

export function capitalizeFirstLetter(string: string) {
  if (!string) return ''
  return string.charAt(0).toUpperCase() + string.slice(1)
}
export const formatAddress = (address?: string) => {
  if (!address) {
    return ''
  }
  let shortAddress = address.substring(0, 6)
  shortAddress += '...'
  shortAddress += address.substring(address.length - 6, address.length)

  return shortAddress
}
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
