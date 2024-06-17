// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox } from '../setup'
import { Args, bcs } from '@/bcs'
import { Transaction } from '@/transactions'
import { fromHEX, toHEX } from '@/utils'
import { BitcoinAddress } from '@/address'

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
    const testAddr = 'bcrt1pwflflg6dz72e8f96f93yzve88yac3nekjl66g52stqauxc5lff6s0peuke'

    const result = await testBox.client.executeViewFunction({
      target: '0x3::address_mapping::resolve_or_generate',
      arguments: [Args.struct(new BitcoinAddress(testAddr).genMultiChainAddress())],
    })

    const expectAddr = '0x57330c8bc64a6068df6a84d3c459e46450b7a15fafae57fe85cb4f95c2ed0198'
    expect(result.return_values![0].decoded_value).eq(expectAddr)
  })
})
