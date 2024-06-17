// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from '@/keypairs'
import { RoochAddress } from './rooch'

describe('address', () => {
  let address: RoochAddress | undefined
  it('should new address with ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()
    address = kp.getPublicKey().toAddress()
    expect(address).toBeDefined()
  })

  it('to hex address', () => {
    expect(address?.toHexAddress()).toBeDefined()
  })

  it('to bech32 address', () => {
    expect(address?.toBech32Address()).toBeDefined()
  })
})
