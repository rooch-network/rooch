// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  JsonRpcProvider,
  Ed25519Keypair,
  PrivateKeyAuth,
  Account,
} from '../../src'
import { RoochServer } from './servers/rooch-server'

describe('SDK', () => {
  let server: RoochServer

  beforeAll(async () => {
    server = new RoochServer()
    //await server.start()
  })

  afterAll(async () => {
    //await server.stop()
  })

  describe('#sessionKey', () => {
    it('Query session keys should be ok', async () => {
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      console.log('roochAddress:', roochAddress)
      console.log(BigInt('0x134afd8'))

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const sessionAccount = await account.createSessionAccount(
        ['0x3::empty::empty', '0x1::*::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // query session Keys
      const sessionKeys = await account.querySessionKeys()
      expect(sessionKeys).toBeDefined()
    })
  })
})
