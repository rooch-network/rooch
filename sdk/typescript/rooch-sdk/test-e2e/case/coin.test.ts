// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { Secp256k1Keypair } from '../../src/keypairs/index.js'
import { Args } from '../../src/bcs/index.js'

describe('Checkpoints Coin API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
  })

  afterAll(async ()=> {
    testBox.cleanEnv()
  })

  it('Cmd publish package should be success', async () => {
    const result = await testBox.cmdPublishPackage('../../../examples/coins', {
      namedAddresses: 'coins=default',
    })

    expect(result).toBeTruthy()
  })

  it('Check balances should be success', async () => {

    const tx = new Transaction()
    tx.callFunction({
      target: `${await testBox.defaultCmdAddress()}::fixed_supply_coin::faucet`,
      args: [
        Args.object({
          address: await testBox.defaultCmdAddress(),
          module: 'fixed_supply_coin',
          name:'Treasury'
        })
      ]
    })

    let result = await testBox.signAndExecuteTransaction(tx)
    expect(result).toBeTruthy()

    await testBox.delay(3)

    let result1 = await testBox.getClient().getBalances({
      owner: testBox.address().toHexAddress(),
      limit: '1',
    })

    expect(result1.has_next_page).toBeTruthy()

    let result2 = await testBox.getClient().getBalances({
      owner: testBox.address().toHexAddress(),
      limit: '1',
      cursor: result1.next_cursor,
    })

    expect(result2.has_next_page).toBeFalsy()
    expect(result2.data.length === 1).toBeTruthy()
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
