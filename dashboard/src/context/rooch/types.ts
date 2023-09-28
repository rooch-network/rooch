// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { JsonRpcProvider, Chain } from '@rooch/sdk'

export type RoochProviderValueType = {
  loading: boolean
  provider: JsonRpcProvider | null
  addChina: (chain: Chain) => Promise<void>
  switchChina: (chain: Chain) => Promise<void>
  switchByChinaId: (chainId: string) => Promise<void>
  deleteChina: (chain: Chain) => Promise<void>
  getAllChina: () => Chain[]
  getActiveChina: () => Chain
}
