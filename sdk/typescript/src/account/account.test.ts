import { describe, it, expect, vi } from 'vitest'
import { ROOCH_DEV_CHIAN_ID } from '../constants'
import { IProvider } from '../provider'
import { Ed25519Keypair } from '../utils/keypairs'
import { Account } from './account'
import { PrivateKeyAuth } from '../auth'

describe('account', () => {
  it('should create Account ok ', async () => {
    const mockProvider: IProvider = {
      getChainId: vi.fn().mockImplementation(() => {
        return ROOCH_DEV_CHIAN_ID
      }),
      getRpcApiVersion: vi.fn(),
      executeViewFunction: vi.fn(),
      sendRawTransaction: vi.fn(),
    }

    const kp = Ed25519Keypair.generate()
    const roochAddress = kp.getPublicKey().toRoochAddress()
    const authorizer = new PrivateKeyAuth(kp)

    const account = new Account(mockProvider, roochAddress, authorizer)
    expect(account).toBeDefined()
  })

  describe('#callFunction', () => {
    it('should execute call function ok', async () => {
      const mockProvider: IProvider = {
        getChainId: vi.fn().mockImplementation(() => {
          return ROOCH_DEV_CHIAN_ID
        }),
        getRpcApiVersion: vi.fn(),
        executeViewFunction: vi.fn(),
        sendRawTransaction: vi.fn(),
      }

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(mockProvider, roochAddress, authorizer)
      expect(account).toBeDefined()

      account.callFunction('0x123::counter::increase', [], [], {
        maxGasAmount: 1000000,
      })
    })
  })
})
