// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from '../keypairs/index.js'
import { RoochAddress, BitcoinAddress, isValidRoochAddress } from '../address/index.js'

const TEST_CASES: { btcAddr: string; roochAddr: string; hexAddr: string }[] = [
  // {
  //   btcAddr: 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke',
  //   roochAddr: '0x57330c8bc64a6068df6a84d3c459e46450b7a15fafae57fe85cb4f95c2ed0198',
  // }
  {
    btcAddr: '18cBEMRxXHqzWWCxZNtU91F5sbUNKhL5PX',
    roochAddr: 'rooch1gxterelcypsyvh8cc9kg73dtnyct822ykx8pmu383qruzt4r93jshtc9fj',
    hexAddr: '0x419791e7f82060465cf8c16c8f45ab9930b3a944b18e1df2278807c12ea32c65',
  },
  {
    btcAddr: 'bc1q262qeyyhdakrje5qaux8m2a3r4z8sw8vu5mysh',
    roochAddr: 'rooch10lnft7hhq37vl0y97lwvkmzqt48fk76y0z88rfcu8zg6qm8qegfqx0rq2h',
    hexAddr: '0x7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12',
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
    const { btcAddr } = TEST_CASES[0]

    const addr = new BitcoinAddress(btcAddr)

    expect(addr).toBeDefined()
  })

  it('To rooch address', () => {
    for (let item of TEST_CASES) {
      const addr = new BitcoinAddress(item.btcAddr)

      const roochAddr = addr.genRoochAddress()

      const genRoochAddr = roochAddr.toBech32Address()
      const genRoochHexAddr = roochAddr.toHexAddress()

      expect(isValidRoochAddress(genRoochAddr)).toBeTruthy()
      expect(isValidRoochAddress(genRoochHexAddr)).toBeTruthy()
      expect(genRoochAddr).eq(item.roochAddr)
      expect(genRoochHexAddr).eq(item.hexAddr)
    }
  })
})
