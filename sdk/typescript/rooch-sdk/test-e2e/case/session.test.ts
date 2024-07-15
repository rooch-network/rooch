// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox } from '../setup.js'
import { bytes } from '../../src/utils/bytes.js'
import { Transaction } from '../../src/transactions/index.js'

describe('Checkpoints Session API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
  })

  it('', () => {
    const aa = '18426974636f696e205369676e6564204d6573736167653a0a353631'
    console.log(bytes('hex', aa))
  })

  it('Create session should be success', async () => {
    const session = await testBox.client.createSession({
      sessionArgs: {
        appName: 'sdk-e2e-test',
        appUrl: 'https://sdk-e2e.com',
        scopes: [
          '0x3::empty::empty_with_signer',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a1b::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a1c::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a11::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a1a::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26aaa::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26aba::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a6a::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a2a::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a3a::*::*',
          '0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a4a::*::*',
        ],
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

  it('Get session keys should be success', async () => {
    const session = await testBox.client.createSession({
      sessionArgs: {
        appName: 'sdk-e2e-test',
        appUrl: 'https://sdk-e2e.com',
        scopes: ['0x3::empty::empty_with_signer'],
      },
      signer: testBox.keypair,
    })

    expect(session).toBeDefined()

    const sessions = await testBox.client.getSessionKeys({
      address: testBox.address().toHexAddress(),
      limit: 10,
    })

    expect(sessions).toBeDefined()
  })
})
