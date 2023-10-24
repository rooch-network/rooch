// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  JsonRpcProvider,
  Ed25519Keypair,
  PrivateKeyAuth,
  Account,
  addressToSeqNumber,
  bcsTypes,
  LocalChain,
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
      const provider = new JsonRpcProvider(LocalChain)
      const result = await provider.executeViewFunction(
        '0x3::account::sequence_number_for_sender',
        [],
        [],
      )
      expect(result).toBeDefined()
    })

    it('view function with serializable arg should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const multiChainIDEther = 60
      const ethAddress = '0xd33293B247A74f9d49c1F6253d909d51242562De'
      const ma = new bcsTypes.MultiChainAddress(
        BigInt(multiChainIDEther),
        addressToSeqNumber(ethAddress),
      )

      const result = await provider.executeViewFunction(
        '0x3::address_mapping::resolve_or_generate',
        [],
        [
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
      )

      expect(result).toBeDefined()
      expect(result.vm_status).toBe('Executed')
      expect(result.return_values).toBeDefined()
    })
  })

  describe('#runFunction', () => {
    it('call function with private key auth should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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

    it('call function with struct be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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

  describe('#getTransactions', () => {
    it('get transaction by index should be ok', async () => {
      /*
      const provider = new JsonRpcProvider()

      const kp = Ed25519Keypair.deriveKeypair(
        'nose aspect organ harbor move prepare raven manage lamp consider oil front',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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

      const result = provider.getTransactionsByHash([tx])
      expect(result).toBeDefined()
      */
    })
  })

  describe('#getStates', () => {
    it('get annotated states should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)
      const result = provider.getStates('/object/0x1')
      expect(result).toBeDefined()
    })
  })

  describe('#sessionKey', () => {
    it('Create session account by registerSessionKey should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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
      expect(session).toBeDefined()

      // run function with sessoin key
      const tx = await sessionAccount.runFunction('0x3::empty::empty', [], [], {
        maxGasAmount: 100000000,
      })

      expect(tx).toBeDefined()
    })

    it('Check session key whether expired should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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

      // check session key expired
      const expired = await account.isSessionKeyExpired(kp2.getPublicKey().toRoochAddress())
      expect(expired).toBeFalsy()

      // run function with sessoin key
      const tx = await sessionAccount.runFunction('0x3::empty::empty', [], [], {
        maxGasAmount: 100000000,
      })

      expect(tx).toBeDefined()
    })

    it('Remove session key should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.deriveKeypair(
        'fiber tube acid imitate frost coffee choose crowd grass topple donkey submit',
      )
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const kp2 = Ed25519Keypair.generate()
      await account.registerSessionKey(
        kp2.getPublicKey().toRoochAddress(),
        ['0x3::empty::empty'],
        100,
      )

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
      expect(session).toBeDefined()
      expect(session.return_values![0].value.value).not.toBe('0x00')

      // run function with sessoin key
      const tx = await account.removeSessionKey(sessionKey)
      expect(tx).toBeDefined()

      // view session Keys
      const session2 = await provider.executeViewFunction(
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

      expect(session2).toBeDefined()
      expect(session2.return_values![0].value.value).toBe('0x00')
    })

    it('Create session account by createSessionAccount should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

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

    it('Create session account with multi scopes should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const sessionAccount = await account.createSessionAccount(
        ['0x3::empty::empty', '0x1::*::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // run function with sessoin key
      const tx = await sessionAccount.runFunction('0x3::empty::empty', [], [], {
        maxGasAmount: 100000000,
      })

      expect(tx).toBeDefined()
    })

    it('Session account runFunction out of score should fail', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      // create session account
      const sessionAccount = await account.createSessionAccount(['0x3::account::*'], 100)
      expect(sessionAccount).toBeDefined()
    })

    it('Query session keys should be ok', async () => {
      const provider = new JsonRpcProvider(LocalChain)

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(provider, roochAddress, authorizer)
      expect(account).toBeDefined()

      //TODO for loop to check the timestamp is updated, or wait for timestamp sync when start rooch server
      // wait timestamp sync
      await new Promise((resolve) => setTimeout(resolve, 10000))

      // create session account
      const sessionAccount = await account.createSessionAccount(
        ['0x3::empty::empty', '0x1::*::*'],
        100,
      )
      expect(sessionAccount).toBeDefined()

      // query session Keys
      const page = await account.querySessionKeys(null, 10)
      expect(page).toBeDefined()
      expect(page.hasNextPage).toBeFalsy()
      expect(page.nextCursor).toBeDefined()
      expect(page.data).toHaveLength(1)
      expect(page.data[0].authentication_key).toBeDefined()
      expect(page.data[0].max_inactive_interval).toBe(100)
      expect(page.data[0].create_time).greaterThan(1696225092)
      expect(page.data[0].last_active_time).greaterThan(1696225092)

      // query next page
      const nextPage = await account.querySessionKeys(page.nextCursor, 10)
      expect(nextPage).toBeDefined()
    })
  })
})
