// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox } from '../setup'
import { Args, bcs } from '@/bcs'
import { Transaction } from '@/transactions'
import { fromHEX, toHEX } from '@/utils'
import { BitcoinAddress } from '@/address'
import { Ed25519Keypair, Secp256k1Keypair } from '@/keypairs'

describe('Checkpoints Reading API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
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

    const account = Secp256k1Keypair.generate()

    const result = await testBox.client.executeViewFunction({
      target: '0x3::address_mapping::resolve_or_generate',
      arguments: [Args.struct(account.getBitcoinAddress().genMultiChainAddress())],
    })

    const expectAddr = account.getRoochAddress().toHexAddress()
    expect(result.return_values![0].decoded_value).eq(expectAddr)
  })
})
