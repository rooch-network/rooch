// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import MetaMaskSDK from '@metamask/sdk'

import { ChainInfo } from '@rooch/sdk'

export type ETHValueType = {
  loading: boolean
  hasProvider: boolean
  provider: MetaMaskSDK | undefined
  chainId: string
  accounts: string[]
  isConnect: boolean
  connect: (china?: ChainInfo) => Promise<void>
  disconnect: () => void
  switchChina: (chain: ChainInfo) => Promise<void>
  addChina: (params: ChainInfo) => Promise<void>
}
