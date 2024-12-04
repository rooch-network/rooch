// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Args } from '../../src/bcs/index.js'
import { Transaction } from '../../src/transactions/index.js'
import { fromHEX } from '../../src/utils/index.js'

describe('Checkpoints Example Entry Function', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
    const result = await testBox.cmdPublishPackage('../../../examples/entry_function_arguments/')

    expect(result).toBeTruthy()
  })

  afterAll(async () => {
    testBox.cleanEnv()
  })

  it('Emit object id should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await testBox.defaultCmdAddress()}::entry_function::emit_object_id`,
      args: [Args.objectId('0x3134')],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()
  })

  it('Call function with object should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await testBox.defaultCmdAddress()}::entry_function::emit_object`,
      args: [
        Args.object({
          address: await testBox.defaultCmdAddress(),
          module: 'entry_function',
          name: 'TestStruct',
        }),
      ],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()
  })

  it('Call function with raw u8 should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await testBox.defaultCmdAddress()}::entry_function::emit_vec_u8`,
      args: [Args.vec('u8', Array.from(fromHEX('0xffff')))],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()
  })
})
