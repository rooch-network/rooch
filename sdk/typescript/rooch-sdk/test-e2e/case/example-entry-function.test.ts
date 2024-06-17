// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox, defaultCmdAddress, cmdPublishPackage, cmd } from '../setup'
import { Args, bcs } from '@/bcs'
import { Transaction } from '@/transactions'
import { fromHEX } from '@/utils'

describe('Checkpoints Example Entry Function', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
  })

  it('Cmd publish package should be success', async () => {
    console.log('publish entry_function_arguments package')
    const result = await cmdPublishPackage('../../../examples/entry_function_arguments/')

    expect(result).toBeTruthy()
  })

  it('Emit object id should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await defaultCmdAddress()}::entry_function::emit_object_id`,
      arguments: [Args.objectId('0x3134')],
    })

    const result = await testBox.client.signAndExecuteTransaction({
      transaction: tx,
      signer: testBox.keypair,
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()
  })

  it('Call function with object should be success', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: `${await defaultCmdAddress()}::entry_function::emit_object`,
      arguments: [
        Args.object({
          address: await defaultCmdAddress(),
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
      target: `${await defaultCmdAddress()}::entry_function::emit_vec_u8`,
      arguments: [Args.vec('u8', Array.from(fromHEX('0xffff')))],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()
  })
})
