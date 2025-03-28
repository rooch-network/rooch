// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { Args } from '../../src/bcs/index.js'

describe('Events API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
    const result = await testBox.cmdPublishPackage('../../../examples/event')
    expect(result).toBeTruthy()
  })

  it('get events should be ok', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await testBox.defaultCmdAddress()}::event_test::emit_event`,
      args: [Args.u64(BigInt(10))],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()

    const tx1 = new Transaction()
    tx1.callFunction({
      target: `${await testBox.defaultCmdAddress()}::event_test::emit_event`,
      args: [Args.u64(BigInt(11))],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()

    const result1 = await testBox.getClient().getEvents({
      eventHandle: `${await testBox.defaultCmdAddress()}::event_test::WithdrawEvent`,
    })

    expect(result1.next_cursor).eq('0')
    expect(result1.data.length).toBeGreaterThan(0)
    expect(result1.has_next_page).eq(false)

    const result2 = await testBox.getClient().queryEvents({
      filter: {
        sender: await testBox.defaultCmdAddress(),
      },
      limit: '1',
    })

    expect(result2.data.length).toBeGreaterThan(0)
    expect(result2.has_next_page).eq(true)
  })
})
