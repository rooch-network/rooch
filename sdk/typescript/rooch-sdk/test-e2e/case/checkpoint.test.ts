// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { Args } from '../../src/bcs/index.js'
import { Secp256k1Keypair } from '../../src/keypairs/index.js'
import { BitcoinAddress } from '../../src/address/index.js'

import { setup, TestBox } from '../setup.js'
import { bytes, str } from '../../src/utils/index.js'
import { fromHEX, toHEX } from '@mysten/bcs'

describe('Checkpoints Reading API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
  })

  it('Get latest sequence number should be success', async () => {
    // 0x2ddb1888a095ba875dc9e9fc939cc69e070fc400faa6d0a47beeae05ef693d50
    //      2ddb1888a095ba875dc9e9fc939cc69e070fc400faa6d0a47beeae05ef693d50

    const s = bytes('utf8', 'tb1q3245npm404htfzvulx6v4w65maqzu6atpxyzja')
    const s1 = toHEX(s)

    console.log(s1)
    // const expectSN = BigInt(0)
    // const resultSN = await testBox.client.getSequenceNumber(testBox.address().toHexAddress())
    //
    // expect(resultSN).eq(expectSN)
  })

  it('Get latest sequence number should be success', async () => {
    const expectSN = BigInt(0)
    const resultSN = await testBox.client.getSequenceNumber(testBox.address().toHexAddress())

    expect(resultSN).eq(expectSN)
  })

  it('Get states should be success', async () => {
    const result = await testBox.client.getStates({
      accessPath: '/object/0x3',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result).toBeDefined()
  })

  it('Get states should be failed', async () => {
    const result = await testBox.client.getStates({
      accessPath: '/object/0x100',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result.length).eq(0)
  })

  it('Resolve rooch address should be success', async () => {

    const caces = [
      'tb1q3245npm404htfzvulx6v4w65maqzu6atpxyzja'
    ]

    const account = Secp256k1Keypair.generate()

    const result = await testBox.client.executeViewFunction({
      target: '0x3::address_mapping::resolve_or_generate',
      args: [Args.struct(account.getBitcoinAddress().genMultiChainAddress())],
    })

    const expectAddr = account.getRoochAddress().toHexAddress()
    expect(result.return_values![0].decoded_value).eq(expectAddr)

    for (let item of caces) {
      const address = new BitcoinAddress(item)

      const result = await testBox.client.executeViewFunction({
        target: '0x3::address_mapping::resolve_or_generate',
        args: [Args.struct(address.genMultiChainAddress())],
      })

      const expectAddr = address.genRoochAddress().toHexAddress()
      expect(result.return_values![0].decoded_value).eq(expectAddr)
    }
  })
})
