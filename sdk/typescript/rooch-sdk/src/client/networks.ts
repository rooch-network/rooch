// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export type NetworkType = 'mainnet' | 'testnet' | 'devnet' | 'localnet'

export function getRoochNodeUrl(network: NetworkType) {
  switch (network) {
    case 'mainnet':
      return 'https://main-seed.rooch.network'
    case 'testnet':
      return 'https://test-seed.rooch.network'
    case 'devnet':
      return 'https://dev-seed.rooch.network:443'
    case 'localnet':
      return 'http://127.0.0.1:6767'
    default:
      throw new Error(`Unknown network: ${network}`)
  }
}
