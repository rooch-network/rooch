import { describe, it, expect, vi } from 'vitest'
import { JsonRpcProvider, Ed25519Keypair, PrivateKeyAuth, Account } from '../../src'

describe('callFunction', () => {
  it('call function with private key auth should be ok', async () => {
    const provider = new JsonRpcProvider()

    const kp = Ed25519Keypair.generate()
    const roochAddress = kp.getPublicKey().toRoochAddress()
    const authorizer = new PrivateKeyAuth(kp)

    const account = new Account(provider, roochAddress, authorizer)
    expect(account).toBeDefined()

    const tx = await account.callFunction('0xbecf2f0f545f16f0c5b69786a4e08a422cec5bf94ffb9ba68e34b730423026ef::counter::increase', [], [])
    expect(tx).toBeDefined()
  })
})
