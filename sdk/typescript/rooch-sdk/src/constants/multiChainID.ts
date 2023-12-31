// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export enum RoochMultiChainID {
  Bitcoin = 0,
  Ether = 60,
  Sui = 784,
  Nostr = 1237,
  Rooch = 20230101,
}

export function RoochMultiChainIDToString(id: RoochMultiChainID) {
  switch (id) {
    case RoochMultiChainID.Ether:
      return 'ether'
    case RoochMultiChainID.Bitcoin:
      return 'bitcoin'
    case RoochMultiChainID.Nostr:
      return 'nostr'
    case RoochMultiChainID.Rooch:
      return 'rooch'
    case RoochMultiChainID.Sui:
      return 'sui'
  }
}
