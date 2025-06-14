// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain } from '../feature/index.js'
import { Wallet, UniSatWallet, OkxWallet, OnekeyWallet } from '../wallet/index.js'
import { LocalWallet } from '../wallet/local.js'

const unisatWallet = new UniSatWallet()
const okxWallet = new OkxWallet()
const onekeyWallet = new OnekeyWallet()
const localWallet = new LocalWallet()
export const supportedWallets = [unisatWallet, okxWallet, onekeyWallet, localWallet]

export async function checkWallets(filter?: SupportChain) {
  const wallets: Wallet[] = supportedWallets.filter(
    (wallet) => wallet.getChain() === filter || !filter,
  )

  return wallets

  // const checkedWallets = await Promise.all(
  //   wallets.map(async (w) => ({
  //     wallet: w,
  //     isInstalled: await w.checkInstalled(),
  //   })),
  // )
  //
  // return checkedWallets
  //   .sort((a, b) => Number(b.isInstalled) - Number(a.isInstalled))
  //   .map((item) => item.wallet)
}
