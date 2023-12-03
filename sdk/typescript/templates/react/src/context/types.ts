// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Ed25519Keypair } from '@roochnetwork/rooch-sdk'

export enum AccountType {
  ETH = 'ETH',
  ROOCH = 'Rooch',
}

export type AccountDataType = {
  roochAddress: string
  address: string
  kp: Ed25519Keypair | null
  activate: boolean
  type: AccountType
}
