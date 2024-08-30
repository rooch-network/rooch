// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'

describe('Checkpoints Transaction API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
  })

  afterAll(async () => {
    testBox.cleanEnv()
  })

  it('Call function with bitcoin auth should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const result = await testBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: testBox.keypair,
    })

    expect(result.execution_info.status.type).eq('executed')
  })

  it('query transactions should be ok', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()

    const result = await testBox.getClient().queryTransactions({
      filter: {
        sender: testBox.address().toHexAddress(),
      },
    })

    expect(result.data.length).toBeGreaterThan(0)
  })
})
