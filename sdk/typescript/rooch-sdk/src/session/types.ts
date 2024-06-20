// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Ed25519Keypair } from '@/keypairs'

export type CreateSessionArgs = {
  appName: string
  appUrl: string
  scopes: string[] | Scope[]
  keypair?: Ed25519Keypair
  maxInactiveInterval?: number
}

export type Scope = {
  address: string
  module: string
  function: string
}
