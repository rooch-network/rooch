// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { address } from '@/types'
import { Ed25519Keypair } from '@/keypairs'

export type CreateSessionArgs = {
  appName: string
  appUrl: string
  scopes: string[] | Scope[]
  keypair?: Ed25519Keypair
  maxInactiveInterval?: number
}

export type Scope = {
  address: address
  module: string
  function: string
}
