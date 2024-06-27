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

  it('Create session should be success', async () => {
    const session = await testBox.client.createSession({
      sessionArgs: {
        appName: 'sdk-e2e-test',
        appUrl: 'https://sdk-e2e.com',
        scopes: ['0x3::empty::empty_with_signer'],
      },
      signer: testBox.keypair,
    })

    const sessions = await testBox.client.getSessionKeys({
      address: testBox.address().toHexAddress(),
    })
    const createdSessionExists = sessions.data.find(
      (item) => item.authenticationKey === session.getAuthKey(),
    )

    expect(createdSessionExists).toBeTruthy()
  })

  it('Check session is expired should be false', async () => {
    const session = await testBox.client.createSession({
      sessionArgs: {
        appName: 'sdk-e2e-test',
        appUrl: 'https://sdk-e2e.com',
        scopes: ['0x3::empty::empty_with_signer'],
      },
      signer: testBox.keypair,
    })

    const result = await testBox.client.sessionIsExpired({
      address: session.getRoochAddress().toHexAddress(),
      authKey: session.getAuthKey(),
    })

    expect(result).toBeFalsy()
  })

  it('Call function with session auth should be success', async () => {
    const session = await testBox.client.createSession({
      sessionArgs: {
        appName: 'sdk-e2e-test',
        appUrl: 'https://sdk-e2e.com',
        scopes: ['0x3::empty::empty_with_signer'],
      },
      signer: testBox.keypair,
    })

    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const result = await testBox.client.signAndExecuteTransaction({
      transaction: tx,
      signer: session,
    })

    expect(result.execution_info.status.type).eq('executed')
  })
})
