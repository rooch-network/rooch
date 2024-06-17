// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function getRoochNodeUrl(network: 'testnet' | 'devnet' | 'localnet') {
  switch (network) {
    case 'testnet':
      return 'https://test-seed.rooch.network'
    case 'devnet':
      return 'https://dev-seed.rooch.network:443'
    case 'localnet':
      return 'http://127.0.0.1:50051'
    default:
      throw new Error(`Unknown network: ${network}`)
  }
}
