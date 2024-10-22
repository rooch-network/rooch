// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain } from '../feature/index.js'
import { Wallet, UniSatWallet, OkxWallet, OnekeyWallet } from '../wellet/index.js'

const unisatWallet = new UniSatWallet()
const okxWallet = new OkxWallet()
const onekeyWallet = new OnekeyWallet()
const supportedWallets = [unisatWallet, okxWallet, onekeyWallet]

export async function checkWallets(filter?: SupportChain) {
  const wallets: Wallet[] = supportedWallets.filter(
    (wallet) => wallet.getChain() === filter || !filter,
  )

  return await Promise.all(wallets.filter(async (w) => await w.checkInstalled()))
}
