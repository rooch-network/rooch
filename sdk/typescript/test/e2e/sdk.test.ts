// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  JsonRpcProvider,
  Ed25519Keypair,
  PrivateKeyAuth,
  Account,
  addressToSeqNumber,
} from '../../src'
import { RoochServer } from './servers/rooch-server'

describe('SDK', () => {
  let server: RoochServer

  beforeAll(async () => {
    server = new RoochServer()
    await server.start()
  })

  afterAll(async () => {
    await server.stop()
  })

  describe('#viewFunction', () => {
    it('view function should be ok', async () => {
      const provider = new JsonRpcProvider()
      const result = await provider.executeViewFunction(
        '0x3::account::sequence_number_for_sender',
        [],
        [],
      )
      expect(result).toBeDefined()
    })
  })

  describe('#runFunction', () => {
    it('call function with private key auth should be ok', async () => {
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      console.log('roochAddress:', roochAddress)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      const tx = await account.runFunction(
        '0x3::account::create_account_entry',
        [],
        [
          {
            type: 'Address',
            value: roochAddress,
          },
        ],
        {
          maxGasAmount: 1000000,
        },
      )

      expect(tx).toBeDefined()
    })
  })

  /*
  describe('#getTransactions', () => {
    it('get transaction by index should be ok', async () => {
      const provider = new JsonRpcProvider()
      const result = provider.getTransactionsByHash([])
      expect(result).toBeDefined()
    })
  })
  */

  describe('#getAnnotatedStates', () => {
    it('get annotated states should be ok', async () => {
      const provider = new JsonRpcProvider()
      const result = provider.getAnnotatedStates('/object/0x1')
      console.log(result)
      expect(result).toBeDefined()
    })
  })

  describe('#sessionKey', () => {
    it('Create session account by registerSessionKey should be ok', async () => {
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      console.log('roochAddress:', roochAddress)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const kp2 = Ed25519Keypair.generate()
      await account.registerSessionKey(
        kp2.getPublicKey().toRoochAddress(),
        ['0x3::empty::empty'],
        100,
      )
      const auth = new PrivateKeyAuth(kp2)
      const sessionAccount = new Account(provider, roochAddress, auth)

      // view session Keys
      const sessionKey = kp2.getPublicKey().toRoochAddress()
      const session = await provider.executeViewFunction(
        '0x3::session_key::get_session_key',
        [],
        [
          {
            type: 'Address',
            value: roochAddress,
          },
          {
            type: { Vector: 'U8' },
            value: addressToSeqNumber(sessionKey),
          },
        ],
      )
      console.log('session:', JSON.stringify(session))

      // run function with sessoin key
      const tx = await sessionAccount.runFunction('0x3::empty::empty', [], [], {
        maxGasAmount: 100000000,
      })

      expect(tx).toBeDefined()
    })

    it('Create session account by createSessionAccount should be ok', async () => {
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      console.log('roochAddress:', roochAddress)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const sessionAccount = await account.createSessionAccount(['0x3::empty::empty'], 100)
      expect(sessionAccount).toBeDefined()

      // run function with sessoin key
      const tx = await sessionAccount.runFunction('0x3::empty::empty', [], [], {
        maxGasAmount: 100000000,
      })

      expect(tx).toBeDefined()
    })

    it('Session account runFunction out of score should fail', async () => {
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      console.log('roochAddress:', roochAddress)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const sessionAccount = await account.createSessionAccount(['0x3::account::*'], 100)
      expect(sessionAccount).toBeDefined()

      expect(async () => {
        // run function out of scope
        await sessionAccount.runFunction('0x3::empty::empty', [], [], {
          maxGasAmount: 100000000,
        })
      }).rejects.toThrow()
    })
  })
})
