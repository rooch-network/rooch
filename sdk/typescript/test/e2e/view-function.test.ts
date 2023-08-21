import { describe, it, expect, vi } from 'vitest'
import { JsonRpcProvider, Ed25519Keypair, PrivateKeyAuth, Account } from '../../src'

describe('viewFunction', () => {
  it('view function should be ok', async () => {
    const provider = new JsonRpcProvider()
    const result = await provider.executeViewFunction('0xbecf2f0f545f16f0c5b69786a4e08a422cec5bf94ffb9ba68e34b730423026ef::counter::value', [], [])
    expect(result).toBeDefined()
  })
})
