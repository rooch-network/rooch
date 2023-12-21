// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochClient, Chain } from '@roochnetwork/rooch-sdk'

export type RoochProviderValueType = {
  loading: boolean
  provider: RoochClient | null
  addChain: (chain: Chain) => Promise<void>
  switchChain: (chain: Chain) => Promise<void>
  switchByChainId: (chainId: string) => Promise<void>
  deleteChain: (chain: Chain) => Promise<void>
  getAllChain: () => Chain[]
  getActiveChain: () => Chain
}
