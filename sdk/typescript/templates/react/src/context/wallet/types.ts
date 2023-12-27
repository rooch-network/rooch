// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import MetaMaskSDK from '@metamask/sdk'

import { ChainInfo } from '@roochnetwork/rooch-sdk'
import { AccountDataType } from '../types'

export type ETHValueType = {
  loading: boolean
  hasProvider: boolean
  provider: MetaMaskSDK | undefined
  chainId: string
  accounts: Map<string, AccountDataType>
  activeAccount: AccountDataType | null
  isConnect: boolean
  connect: (chain?: ChainInfo) => Promise<void>
  sendTransaction: (params: any[]) => Promise<any>
  waitTxConfirmed: (txHash: string) => Promise<any>
  disconnect: () => void
  switchChain: (chain: ChainInfo) => Promise<void>
  addChain: (params: ChainInfo) => Promise<void>
}
