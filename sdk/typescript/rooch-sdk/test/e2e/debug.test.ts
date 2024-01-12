// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'

import { RoochClient, Ed25519Keypair, PrivateKeyAuth, Account, LocalChain } from '../../src'
import { RoochServer } from './servers/rooch-server'
import { RoochCli } from './cli/rooch-cli'

describe('SDK', () => {
  let server: RoochServer
  let cli: RoochCli
  let defaultAddress: string

  beforeAll(async () => {
    // start rooch server
    server = new RoochServer()
    await server.start()

    // deploy example app
    cli = new RoochCli()

    await cli.execute([
      'move',
      'publish',
      '-p',
      '../../../examples/entry_function_arguments/',
      '--named-addresses',
      'rooch_examples=default',
    ])

    defaultAddress = await cli.defaultAccountAddress()
  })

  afterAll(async () => {
    await server.stop()
  })

  describe('#debug', () => {
    it('Test', async () => {
      expect(true).toBeTruthy()
    })

    it('call function with objectid be ok', async () => {
      const client = new RoochClient(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(client, roochAddress, authorizer)
      expect(account).toBeDefined()

      const tx = await account.runFunction(
        `${defaultAddress}::entry_function::emit_object_id`,
        [],
        [
          {
            type: 'ObjectID',
            value: '0x3134',
          },
        ],
        {
          maxGasAmount: 2000000,
        },
      )

      expect(tx).toBeDefined()
    })
  })
})
