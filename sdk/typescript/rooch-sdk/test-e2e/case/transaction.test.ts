// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'

describe('Checkpoints Transaction API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
  })

  it('Call function with bitcoin auth should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const result = await testBox.client.signAndExecuteTransaction({
      transaction: tx,
      signer: testBox.keypair,
    })

    expect(result.execution_info.status.type).eq('executed')
  })
})
