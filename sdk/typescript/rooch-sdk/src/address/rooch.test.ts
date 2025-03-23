// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { Ed25519Keypair } from '../keypairs/index.js'
import { RoochAddress } from './rooch.js'
import { normalizeRoochAddress } from './util.js'
import { bech32m } from 'bech32'
import { ROOCH_BECH32_PREFIX } from './address.js'
import { fromHEX } from '../utils/hex.js'

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

describe('RoochAddress', () => {
  it('should create RoochAddress when given a valid hex string', () => {
    const validHex = '0x1234567890abcdef'
    const address = new RoochAddress(validHex)
    expect(address).toBeDefined()
    expect(address.toHexAddress()).toBe(normalizeRoochAddress(validHex))
    expect(address.toBech32Address()).toBe(
      bech32m.encode(
        ROOCH_BECH32_PREFIX,
        bech32m.toWords(fromHEX(normalizeRoochAddress(validHex))),
      ),
    )
  })

  it('should throw error when given an invalid hex string', () => {
    const invalidHex = '0x12345G'
    expect(() => new RoochAddress(invalidHex)).toThrow()
  })
})
