// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest'
import { setup, TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'

describe('Checkpoints Session API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = await setup()
  })

  /// session scope
  /// 通用 args，
  it('Create session should be success', async () => {
    const info = {
      title: 'Welcome to rooch-portal\nYou will authorize session:',
      target: '0x3::session::create_with_xxx',
      args: ['0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::*::*'],
      authkey: '',
      timeOut: 1200,
      hash: 'Rooch Transaction:\n3ec44534d5e4aeeb7ae4806e91ed69b78f2b9ea1cd922fbc5093abada644407a',
    }

    // hash 合约拼接，无须验证与传递

    // timeOut 数字限制
    // scope 与参数对比？ 格式验证

    // title 如何限制？

    console.log(info)

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
