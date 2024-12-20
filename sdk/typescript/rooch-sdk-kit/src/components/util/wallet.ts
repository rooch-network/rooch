// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { NetworkType } from '@roochnetwork/rooch-sdk'
import { Wallet, WalletNetworkType } from '../../wellet/index.js'

const NETWORK_MAP: Record<NetworkType, WalletNetworkType | undefined> = {
  mainnet: 'livenet',
  testnet: 'testnet',
  devnet: undefined,
  localnet: undefined,
}
export const checkWalletNetwork = async (wallet: Wallet, roochNetwork: NetworkType) => {
  try {
    const walletNetwork = await wallet.getNetwork()
    const target = NETWORK_MAP[roochNetwork]
    if (target && walletNetwork !== target) {
      return target
    }
  } catch (_) {}
  return undefined
}
