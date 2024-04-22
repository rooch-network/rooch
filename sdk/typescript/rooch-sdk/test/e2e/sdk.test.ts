// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { arrayify } from '@ethersproject/bytes'
import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  RoochClient,
  RoochAccount,
  addressToSeqNumber,
  bcs,
  LocalNetwork,
  RoochSessionAccount,
} from '../../src'
import { RoochServer } from './servers/rooch-server'
import { RoochCli } from './cli/rooch-cli'

describe('SDK', () => {
  let server: RoochServer
  let cli: RoochCli
  let defaultAddress: string
  let client: RoochClient

  beforeAll(async () => {
    // start rooch server
    server = new RoochServer()
    await server.start()

    cli = new RoochCli()

    // deploy example entry_function_arguments
    await cli.execute([
      'move',
      'publish',
      '-p',
      '../../../examples/entry_function_arguments/',
      '--named-addresses',
      'rooch_examples=default',
    ])

    // deploy example counter
    await cli.execute([
      'move',
      'publish',
      '-p',
      '../../../examples/counter/',
      '--named-addresses',
      'rooch_examples=default',
    ])

    // deploy example coins
    await cli.execute([
      'move',
      'publish',
      '-p',
      '../../../examples/coins/',
      '--named-addresses',
      'coins=default',
    ])

    defaultAddress = await cli.defaultAccountAddress()
    client = new RoochClient(LocalNetwork)
  })

  afterAll(async () => {
    await server.stop()
  })

  describe('#viewFunction', () => {
    it('view function should be ok', async () => {
      const account = new RoochAccount(client)
      const number = await client.getSequenceNumber(account.getAddress())
      expect(number).toEqual('0')
    })

    it('view function with serializable arg should be ok', async () => {
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
      const account = new RoochAccount(client)
      expect(account).toBeDefined()

      const tx = await account.executeTransaction('0x3::account::create_account_entry', [
        {
          type: 'Address',
          value: account.getAddress(),
        },
      ])

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('call function with objectid be ok', async () => {
      const account = new RoochAccount(client)
      expect(account).toBeDefined()

      const tx = await account.executeTransaction(
        `${defaultAddress}::entry_function::emit_object_id`,
        [
          {
            type: 'ObjectID',
            value: '0x3134',
          },
        ],
      )

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('call function with object be ok', async () => {
      const account = new RoochAccount(client)
      const tx = await account.executeTransaction(
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
      )

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('call function with raw be ok', async () => {
      const account = new RoochAccount(client)
      const tx = await account.executeTransaction(
        `${defaultAddress}::entry_function::emit_vec_u8`,
        [
          {
            type: 'Raw',
            value: arrayify('0xffff'),
          },
        ],
      )

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('call fixed_supply_coin::faucet be ok', async () => {
      const account = new RoochAccount(client)

      const tx = await account.executeTransaction(`${defaultAddress}::fixed_supply_coin::faucet`, [
        {
          type: 'Object',
          value: {
            address: defaultAddress,
            module: 'fixed_supply_coin',
            name: 'Treasury',
          },
        },
      ])
      expect(tx.execution_info.status.type).toBe('executed')

      const fscBalance = await account.getBalance(`${defaultAddress}::fixed_supply_coin::FSC`)
      expect(fscBalance.balance).toBe('10000')
    })
  })

  describe('#getTransactions', () => {
    it('get transaction by index should be ok', async () => {
      const account = new RoochAccount(client)
      const tx = await account.executeTransaction('0x3::account::create_account_entry', [
        {
          type: 'Address',
          value: account.getAddress(),
        },
      ])

      expect(tx.execution_info.status.type).toBe('executed')

      const result = client.getTransactionsByHashes([tx.execution_info.tx_hash])
      expect(result).toBeDefined()
    })
  })

  describe('#getStates', () => {
    it('get annotated states should be ok', async () => {
      const result = client.getStates({ accessPath: '/object/0x1' })
      expect(result).toBeDefined()
    })
  })

  describe('#sessionAccount', () => {
    it('Create session account should be ok', async () => {
      const account = new RoochAccount(client)
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        account,
        'test',
        'test',
        ['0x3::empty::empty'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // view session Key
      const session = await sessionAccount.getSessionKey()
      expect(session).toBeDefined()

      // run function with sessoin key
      const tx = await sessionAccount.executeTransaction('0x3::empty::empty')

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('Check session key whether expired should be ok', async () => {
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        new RoochAccount(client),
        'test',
        'test',
        ['0x3::empty::empty'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // check session key expired
      const expired = await sessionAccount.isExpired()
      expect(expired).toBeFalsy()

      // run function with session
      const tx = await sessionAccount.executeTransaction('0x3::empty::empty', [], [])

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('Remove session key should be ok', async () => {
      // create session key
      const account = new RoochAccount(client)
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        account,
        'test',
        'test',
        ['0x3::empty::empty'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // view session key
      const sessionKey = await sessionAccount.getSessionKey()
      expect(sessionKey).toBeDefined()

      // destroy session
      await sessionAccount.destroy()

      // view session key
      const sessionKey2 = await sessionAccount.getSessionKey()
      expect(sessionKey2).toBeNull()
    })

    it('Create session account with multi scopes should be ok', async () => {
      const account = new RoochAccount(client)
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        account,
        'test',
        'test',
        ['0x3::empty::empty', '0x1::*::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // run function with session
      const tx = await sessionAccount.executeTransaction('0x3::empty::empty')

      expect(tx.execution_info.status.type).toBe('executed')
    })

    it('Session account runFunction out of score should fail', async () => {
      const account = new RoochAccount(client)
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        account,
        'test',
        'test',
        ['0x2::account::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      try {
        await sessionAccount.executeTransaction('0x3::empty::empty')
      } catch (e) {
        expect(e).toBeDefined()
      }
    })

    it('Query session keys should be ok', async () => {
      const account = new RoochAccount(client)
      const sessionAccount = await RoochSessionAccount.CREATE(
        client,
        account,
        'test',
        'test',
        ['0x3::empty::empty', '0x1::*::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // wait timestamp sync
      // await new Promise((resolve) => setTimeout(resolve, 10000))

      // query session Keys
      const page = await sessionAccount.querySessionKeys()

      console.log(page)

      expect(page).toBeDefined()
      expect(page.hasNextPage).toBeFalsy()
      expect(page.nextCursor).toBeDefined()
      expect(page.data).toHaveLength(1)
      expect(page.data[0].authenticationKey).toBeDefined()
      expect(page.data[0].maxInactiveInterval).toBe(100)
      expect(page.data[0].createTime).toBe(0)
      expect(page.data[0].lastActiveTime).toBe(0)
    })
  })
})
