// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { Args } from '../../src/bcs/index.js'
import { Secp256k1Keypair } from '../../src/keypairs/index.js'
import { BitcoinAddress, BitcoinNetowkType } from '../../src/address/index.js'
import { Transaction } from '../../src/transactions/index.js'
import { TARGETED_RPC_VERSION } from '../../src/version.js'

import { TestBox } from '../setup.js'

describe('Checkpoints Reading API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
  })

  afterAll(async () => {
    testBox.cleanEnv()
  })

  it('Get latest rpc version eq local version', async () => {
    const resultSN = await testBox.getClient().getRpcApiVersion()

    expect(resultSN).eq(TARGETED_RPC_VERSION)
  })

  it('Get latest sequence number should be success', async () => {
    const expectSN = BigInt(0)
    const resultSN = await testBox.getClient().getSequenceNumber(testBox.address().toHexAddress())

    expect(resultSN).eq(expectSN)
  })

  it('Get states should be success', async () => {
    const result = await testBox.getClient().getStates({
      accessPath: '/object/0x3',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result).toBeDefined()
  })

  it('Get states should be failed', async () => {
    const result = await testBox.getClient().getStates({
      accessPath: '/object/0x100',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result.length).eq(0)
  })

  // it('Resolve rooch address should be success', async () => {
  //   const caces = ['tb1q3245npm404htfzvulx6v4w65maqzu6atpxyzja']
  //
  //   const account = Secp256k1Keypair.generate()
  //
  //   const result = await testBox.getClient().executeViewFunction({
  //     target: '0x3::address_mapping::resolve_or_generate',
  //     args: [Args.struct(account.getBitcoinAddress().genMultiChainAddress())],
  //   })
  //
  //   const expectAddr = account.getRoochAddress().toHexAddress()
  //   expect(result.return_values![0].decoded_value).eq(expectAddr)
  //
  //   for (let item of caces) {
  //     const address = new BitcoinAddress(item)
  //
  //     const result = await testBox.getClient().executeViewFunction({
  //       target: '0x3::address_mapping::resolve_or_generate',
  //       args: [Args.struct(address.genMultiChainAddress())],
  //     })
  //
  //     const expectAddr = address.genRoochAddress().toHexAddress()
  //     expect(result.return_values![0].decoded_value).eq(expectAddr)
  //   }
  // })

  it('resolve btc address should be ok', async () => {
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const result = await testBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: testBox.keypair,
    })

    expect(result.execution_info.status.type).eq('executed')

    const result1 = await testBox.getClient().resolveBTCAddress({
      roochAddress: testBox.keypair.getRoochAddress(),
      network: BitcoinNetowkType.Testnet,
    })

    expect(result1?.toStr()).eq(testBox.keypair.getBitcoinAddress().toStr())
  })

  it('get states should be ok', async () => {
    const result = await testBox.getClient().getStates({
      accessPath: '/object/0x3',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result).toBeDefined()
    expect(result.length).toBeGreaterThan(0)
  })

  it('list states should be ok', async () => {
    const result = await testBox.getClient().listStates({
      accessPath: '/resource/0x3',
      stateOption: {
        decode: true,
        showDisplay: true,
      },
    })

    expect(result.data.length).toBeGreaterThan(0)
  })

  it('get events should be ok', async () => {
    const result = await testBox.cmdPublishPackage('../../../examples/event')

    expect(result).toBeTruthy()

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

    await testBox.delay(3)

    const result1 = await testBox.getClient().getEvents({
      eventHandleType: `${await testBox.defaultCmdAddress()}::event_test::WithdrawEvent`,
      limit: '1',
      descendingOrder: false,
    })

    expect(result1.next_cursor).eq('0')
    expect(result1.data.length).toBeGreaterThan(0)
    expect(result1.has_next_page).eq(true)

    const result2 = await testBox.getClient().queryEvents({
      filter: {
        sender: await testBox.defaultCmdAddress(),
      },
      limit: '1',
    })

    expect(result2.data.length).toBeGreaterThan(0)
    expect(result2.has_next_page).eq(true)
  })

  it('query object states should be ok', async () => {
    const result = await testBox.getClient().queryObjectStates({
      filter: {
        owner: '0x3',
      },
    })

    expect(result.data.length).toBeGreaterThan(0)
  })
})
