// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { JsonRpcProvider, Ed25519Keypair, PrivateKeyAuth, Account } from '../../src'
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

      console.log('tx:', tx)

      expect(tx).toBeDefined()
    })
  })
})
