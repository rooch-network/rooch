// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { UniSatWallet } from './unisat.js'
import { Wallet } from '../wellet/wallet.js'

let wallets: Wallet[] = [new UniSatWallet(false)]

export function getWallets() {
  return wallets
}

export function registerMock(mockWallets: Wallet) {
  wallets.push(mockWallets)
}
