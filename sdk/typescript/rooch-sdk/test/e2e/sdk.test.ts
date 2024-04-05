// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { arrayify } from '@ethersproject/bytes'
import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  RoochClient,
  Ed25519Keypair,
  PrivateKeyAuth,
  RoochAccount,
  addressToSeqNumber,
  bcs,
  LocalNetwork, RoochSessionAccount,
} from '../../src'
import { RoochServer } from './servers/rooch-server'
import { RoochCli } from './cli/rooch-cli'

describe('SDK', () => {
  let server: RoochServer
  let cli: RoochCli
  let defaultAddress: string

  beforeAll(async () => {
    // start rooch server
    // server = new RoochServer()
    // await server.start()
    //
    cli = new RoochCli()
    //
    // // deploy example entry_function_arguments
    // await cli.execute([
    //   'move',
    //   'publish',
    //   '-p',
    //   '../../../examples/entry_function_arguments/',
    //   '--named-addresses',
    //   'rooch_examples=default',
    // ])
    //
    // // deploy example counter
    // await cli.execute([
    //   'move',
    //   'publish',
    //   '-p',
    //   '../../../examples/counter/',
    //   '--named-addresses',
    //   'rooch_examples=default',
    // ])
    //
    // // deploy example coins
    // await cli.execute([
    //   'move',
    //   'publish',
    //   '-p',
    //   '../../../examples/coins/',
    //   '--named-addresses',
    //   'coins=default',
    // ])

    defaultAddress = await cli.defaultAccountAddress()
  })

  afterAll(async () => {
    await server.stop()
  })

  describe('#viewFunction', () => {
    it('view function should be ok', async () => {
      const client = new RoochClient(LocalNetwork)
      const result = await client.executeViewFunction({
        funcId: '0x2::account::sequence_number_for_sender',
      })
      expect(result).toBeDefined()
    })

    it('view function with serializable arg should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const multiChainIDEther = 60
      const ethAddress = '0xd33293B247A74f9d49c1F6253d909d51242562De'
      const ma = new bcs.MultiChainAddress(
        BigInt(multiChainIDEther),
        addressToSeqNumber(ethAddress),
      )

      const result = await client.executeViewFunction({
        funcId: '0x3::address_mapping::resolve_or_generate',
        tyArgs: [],
        args: [
          {
            type: {
              Struct: {
                address: '0x3',
                module: 'address_mapping',
                name: 'MultiChainAddress',
              },
            },
            value: ma,
          },
        ],
      })

      expect(result).toBeDefined()
      expect(result.vm_status).toBe('Executed')
      expect(result.return_values).toBeDefined()
    })
  })

  describe('#sendTransaction', () => {
    it('call function with private key auth should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        '0x3::account::create_account_entry',
        [
          {
            type: 'Address',
            value: await account.getRoochAddress(),
          },
        ],
        [],
        {
          maxGasAmount: 100000000,
        },
      )

      expect(tx).toBeDefined()
    })

    it('call function with objectid be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )

      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        `${defaultAddress}::entry_function::emit_object_id`,
        [
          {
            type: 'ObjectID',
            value: '0x3134',
          },
        ],
        [],
        {
          maxGasAmount: 2000000,
        },
      )

      expect(tx).toBeDefined()
    })

    it('call function with object be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )

      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        `${defaultAddress}::entry_function::emit_object`,
        [
          {
            type: 'Object',
            value: {
              address: defaultAddress,
              module: 'entry_function',
              name: 'TestStruct',
            },
          },
        ],
        [],
        {
          maxGasAmount: 2000000,
        },
      )

      expect(tx).toBeDefined()
    })

    it('call function with raw be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )

      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        `${defaultAddress}::entry_function::emit_vec_u8`,
        [
          {
            type: 'Raw',
            value: arrayify('0xffff'),
          },
        ],
        [],
        {
          maxGasAmount: 2000000,
        },
      )

      expect(tx).toBeDefined()
    })

    it('call fixed_supply_coin::faucet be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.generate()

      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        `${defaultAddress}::fixed_supply_coin::faucet`,
        [
          {
            type: 'Object',
            value: {
              address: defaultAddress,
              module: 'fixed_supply_coin',
              name: 'Treasury',
            },
          },
        ],
        [],
        {
          maxGasAmount: 200000000,
        },
      )

      expect(tx).toBeDefined()

      const fscBalance = await account.getBalance(`${defaultAddress}::fixed_supply_coin::FSC`)
      expect(fscBalance.balance).toBe('10000')
    })
  })

  describe('#getTransactions', () => {
    it('get transaction by index should be ok', async () => {

      const client = new RoochClient()

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const account = new RoochAccount(client, kp)
      expect(account).toBeDefined()

      const tx = await account.sendTransaction(
        '0x3::account::create_account_entry',
        [
          {
            type: 'Address',
            value: await account.getRoochAddress(),
          },
        ],
        [],
        {
          maxGasAmount: 2000000,
        },
      )

      expect(tx).toBeDefined()

      const result = client.getTransactionsByHashes([tx])
      expect(result).toBeDefined()
    })
  })

  describe('#getStates', () => {
    it('get annotated states should be ok', async () => {
      const client = new RoochClient(LocalNetwork)
      const result = client.getStates('/object/0x1')
      expect(result).toBeDefined()
    })
  })

  describe('#sessionKey', () => {
    it('Create session account should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )

      const account = new RoochAccount(client, kp)

      const sessionAccount = await RoochSessionAccount.CREATE(client, account, ['0x3::empty::empty'], 1000)
      expect(sessionAccount).toBeDefined()

      // view session Keys
      const sessionA = await client.executeViewFunction({
        funcId: '0x3::session_key::get_session_key',
        tyArgs: [],
        args: [
          {
            type: 'Address',
            value: await sessionAccount.getRoochAddress(),
          },
          {
            type: { Vector: 'U8' },
            value: addressToSeqNumber(await sessionAccount.getAuthKey()),
          },
        ],
      })
      expect(sessionA).toBeDefined()
      const session = await sessionAccount.querySessionKeys(null, 10)

      console.log(session)

      // run function with sessoin key
      const tx = await sessionAccount.sendTransaction('0x3::empty::empty', [], [], {
        maxGasAmount: 200000000,
      })

      expect(tx).toBeDefined()
    })

    it('Check session key whether expired should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const sessionAccount = await RoochSessionAccount.CREATE(client, new RoochAccount(client, kp), ['0x3::empty::empty'], 100 )
      expect(sessionAccount).toBeDefined()

      // check session key expired
      const expired = await sessionAccount.isExpired()
      expect(expired).toBeFalsy()

      // run function with sessoin key
      const tx = await sessionAccount.sendTransaction('0x3::empty::empty', [], [], {
        maxGasAmount: 200000000,
      })

      expect(tx).toBeDefined()
    })

    it('Remove session key should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )

      const account = new RoochAccount(client, kp)

      const sessionAccount = await RoochSessionAccount.CREATE(client, account, ['0x3::empty::empty'], 100)
      expect(sessionAccount).toBeDefined()

      // view session Keys
      const sessionKey = await sessionAccount.getAuthKey()
      const session = await client.executeViewFunction({
        funcId: '0x3::session_key::get_session_key',
        tyArgs: [],
        args: [
          {
            type: 'Address',
            value: await sessionAccount.getRoochAddress(),
          },
          {
            type: { Vector: 'U8' },
            value: addressToSeqNumber(sessionKey),
          },
        ],
      })
      expect(session).toBeDefined()
      expect(session.return_values![0].value.value).not.toBe('0x00')

      // run function with sessoin key
      const tx = await sessionAccount.destroy()
      expect(tx).toBeDefined()

      // view session Keys
      const session2 = await client.executeViewFunction({
        funcId: '0x3::session_key::get_session_key',
        tyArgs: [],
        args: [
          {
            type: 'Address',
            value: await sessionAccount.getRoochAddress(),
          },
          {
            type: { Vector: 'U8' },
            value: addressToSeqNumber(sessionKey),
          },
        ],
      })

      expect(session2).toBeDefined()
      expect(session2.return_values![0].value.value).toBe('0x00')
    })

    it('Create session account with multi scopes should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.generate()

      const sessionAccount = await RoochSessionAccount.CREATE(client, new RoochAccount(client, kp), ['0x3::empty::empty', '0x1::*::*'], 100)
      expect(sessionAccount).toBeDefined()

      // run function with sessoin key
      const tx = await sessionAccount.sendTransaction('0x3::empty::empty', [], [], {
        maxGasAmount: 200000000,
      })

      expect(tx).toBeDefined()
    })

    it('Session account runFunction out of score should fail', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.generate()

      const sessionAccount =await RoochSessionAccount.CREATE(client, new RoochAccount(client, kp), ['0x2::account::*'], 100)
      expect(sessionAccount).toBeDefined()

      const tx = await sessionAccount.sendTransaction('0x3::empty::empty', [], [], {
        maxGasAmount: 200000000,
      })

      expect(tx).toBeDefined()
    })

    it('Query session keys should be ok', async () => {
      const client = new RoochClient(LocalNetwork)

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const sessionAccount = await RoochSessionAccount.CREATE(client, new RoochAccount(client, kp),['0x3::empty::empty', '0x1::*::*'], 100 )
      expect(sessionAccount).toBeDefined()

      // wait timestamp sync
      await new Promise((resolve) => setTimeout(resolve, 10000))

      // query session Keys
      const page = await sessionAccount.querySessionKeys(null, 10)
      expect(page).toBeDefined()
      expect(page.hasNextPage).toBeFalsy()
      expect(page.nextCursor).toBeDefined()
      expect(page.data).toHaveLength(1)
      expect(page.data[0].authentication_key).toBeDefined()
      expect(page.data[0].max_inactive_interval).toBe(100)
      expect(page.data[0].create_time).greaterThan(1696225092)
      expect(page.data[0].last_active_time).greaterThan(1696225092)
    })
  })
})
