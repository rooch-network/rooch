import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import {
  JsonRpcProvider,
  Ed25519Keypair,
  PrivateKeyAuth,
  Account,
} from '../../src'
import { RoochServer } from './servers/rooch-server'

describe('callFunction', () => {
  let server: RoochServer

  beforeAll(async () => {
    server = new RoochServer()
    await server.start()
  })

  afterAll(async () => {
    await server.stop()
  })

  it('call function with private key auth should be ok', async () => {
    const provider = new JsonRpcProvider()

    const kp = Ed25519Keypair.generate()
    const roochAddress = kp.getPublicKey().toRoochAddress()
    const authorizer = new PrivateKeyAuth(kp)

    const account = new Account(provider, roochAddress, authorizer)
    expect(account).toBeDefined()

    const tx = await account.callFunction(
      '0x1::account::create_account_entry',
      [],
      [
        {
          type: 'Address',
          value: roochAddress,
        },
      ],
    )

    expect(tx).toBeDefined()
  })
})
