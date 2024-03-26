// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SupportChain } from '../feature'
import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'

export function chain2MultiChainID(chain: SupportChain) {
  switch (chain) {
    case SupportChain.BITCOIN:
      return RoochMultiChainID.Bitcoin
    case SupportChain.ETH:
      return RoochMultiChainID.Ether
    case SupportChain.Rooch:
      return RoochMultiChainID.Rooch
  }
}

// rooch 多链

// rooch 多网络

