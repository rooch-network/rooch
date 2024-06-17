// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from '@/keypairs'
import { RoochAddress } from '@/address/rooch.ts'
import { BitcoinAddress } from '@/address/bitcoin.ts'

describe('bitcoin address', () => {
  let address: RoochAddress | undefined
  it('should new address with ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()
    address = kp.getPublicKey().toAddress()
    expect(address).toBeDefined()
  })

  it('from address', () => {
    const testBitcoinAddr = 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke'

    const addr = new BitcoinAddress(testBitcoinAddr)

    expect(addr).toBeDefined()
  })

  it('to rooch address', () => {
    const testBitcoinAddr = 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke'

    const addr = new BitcoinAddress(testBitcoinAddr)
    const resultRoochAddr = addr.genRoochAddress()

    const expectRoochAddr = '0x57330c8bc64a6068df6a84d3c459e46450b7a15fafae57fe85cb4f95c2ed0198'

    expect(resultRoochAddr.toHexAddress()).eq(expectRoochAddr)
  })

  it('to bech32 address', () => {
    const testBitcoinAddr = 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke'

    const addr = new BitcoinAddress(testBitcoinAddr)
    const resultRoochAddr = addr.genRoochAddress()

    const expectRoochAddr = 'rooch12uesez7xffsx3hm2snfugk0yv3gt0g2l47h90l59ed8etshdqxvqv4yw2t'

    expect(resultRoochAddr.toBech32Address()).eq(expectRoochAddr)
  })
})
