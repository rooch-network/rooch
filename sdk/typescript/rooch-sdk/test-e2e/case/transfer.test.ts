// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { Secp256k1Keypair } from '../../src/keypairs/index.js'
import { Args } from '../../src/bcs/index.js'

describe('Checkpoints Transfer API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
  })

  afterAll(async ()=> {
    testBox.cleanEnv()
  })

  it('Transfer gas coin should be success', async () => {
    const amount = BigInt(10000000)
    const coinType = '0x3::gas_coin::GasCoin'
    const [sender, recipient] = [testBox.keypair, Secp256k1Keypair.generate()]

    // get gas
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::gas_coin::faucet_entry',
      args: [Args.u256(BigInt(10000000000))],
    })

    expect(await testBox.signAndExecuteTransaction(tx)).toBeTruthy()

    // transfer
    const transferResult = await testBox.getClient().transfer({
      signer: sender,
      recipient: recipient.getRoochAddress(),
      amount: amount,
      coinType: {
        target: coinType,
      },
    })

    expect(transferResult.execution_info.status.type === 'executed').toBeTruthy()

    await testBox.delay(3)

    // check balance
    const recipientBalance = await testBox.getClient().getBalance({
      owner: recipient.getRoochAddress().toHexAddress(),
      coinType,
    })

    expect(BigInt(recipientBalance.balance)).eq(amount)
  })
})
