// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { Unsubscribe } from '../../src/client/client.js'
import { Args } from '../../src/index.js'

describe('WebSocket Transport Tests', () => {
  let wsTestBox: TestBox

  beforeAll(async () => {
    wsTestBox = TestBox.setup()
    const result = await wsTestBox.cmdPublishPackage('../../../examples/event')
    expect(result).toBeTruthy()
  })

  afterAll(async () => {
    await wsTestBox.cleanEnv()
  })

  it('should execute single transaction successfully', async () => {
    let unsubscribe: Unsubscribe

    const eventReceived = new Promise<boolean>((resolve) => {
      wsTestBox
        .getClient()
        .subscribeEvent({
          onMessage(event) {
            if (event.event_type.includes('event_test::WithdrawEvent')) {
              unsubscribe()
              resolve(true)
            }
          },
        })
        .then((sub) => {
          unsubscribe = sub
        })
        .catch((err) => {
          console.log(err)
          resolve(false)
        })
    })

    await new Promise((resolve) => setTimeout(resolve, 2000))

    const cmdAddress = await wsTestBox.defaultCmdAddress()

    // Execute a transaction that emits an event
    const tx = new Transaction()
    const target = `${cmdAddress}::event_test::emit_event`

    tx.callFunction({
      target: target,
      args: [Args.u64(BigInt(10))],
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    expect(txResult.execution_info.status.type).eq('executed')

    const result = await Promise.race([
      eventReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          resolve(false)
        }, 15000),
      ),
    ])

    expect(result).toBe(true)
  })
})
