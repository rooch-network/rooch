// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from '@/keypairs/ed25519/keypair.js'
import { RoochAddress } from '@/address/rooch'
import { BitcoinAddress } from '@/address/bitcoin'
import { isValidRoochAddress } from '@/address/util'

const TEST_CASES: [{ btcAddr: string; roochAddr: string }] = [
  {
    btcAddr: 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke',
    roochAddr: '0x57330c8bc64a6068df6a84d3c459e46450b7a15fafae57fe85cb4f95c2ed0198',
  },
]

describe('Bitcoin address', () => {
  let address: RoochAddress | undefined

  it('New address with ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()
    address = kp.getPublicKey().toAddress()
    expect(address).toBeDefined()
  })

  it('From address', () => {
    const {btcAddr} = TEST_CASES[0]

    const addr = new BitcoinAddress(btcAddr)

    expect(addr).toBeDefined()
  })

  it('To rooch address', () => {
    const {btcAddr, roochAddr} = TEST_CASES[0]

    const addr = new BitcoinAddress(btcAddr)
    const resultRoochAddr = addr.genRoochAddress()

    expect(isValidRoochAddress(resultRoochAddr.toBech32Address())).toBeTruthy()
    expect(isValidRoochAddress(resultRoochAddr.toHexAddress())).toBeTruthy()
    expect(resultRoochAddr.toHexAddress()).eq(roochAddr)
  })
})
