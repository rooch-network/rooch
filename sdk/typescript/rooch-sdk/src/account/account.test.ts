// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi } from 'vitest'
import { DevNetwork } from '../constants'
import { IClient } from '../client'
import { Ed25519Keypair } from '../utils/keypairs'
import { Account } from './account'
import { PrivateKeyAuth } from '../auth'

describe('account', () => {
  it('should create Account ok ', async () => {
    const mockProvider: IClient = {
      getChainId: vi.fn().mockImplementation(() => {
        return DevNetwork.id
      }),
      getRpcApiVersion: vi.fn(),
      executeViewFunction: vi.fn(),
      sendRawTransaction: vi.fn(),
      getStates: vi.fn(),
      listStates: vi.fn(),
    }

    const kp = Ed25519Keypair.generate()
    const roochAddress = kp.getPublicKey().toRoochAddress()
    const authorizer = new PrivateKeyAuth(kp)

    const account = new Account(mockProvider, roochAddress, authorizer)
    expect(account).toBeDefined()
  })

  describe('#runFunction', () => {
    it('should execute call function ok', async () => {
      const mockProvider: IClient = {
        getChainId: vi.fn().mockImplementation(() => {
          return DevNetwork.id
        }),
        getRpcApiVersion: vi.fn(),
        executeViewFunction: vi.fn(),
        sendRawTransaction: vi.fn(),
        getStates: vi.fn(),
        listStates: vi.fn(),
      }

      const kp = Ed25519Keypair.generate()
      const roochAddress = kp.getPublicKey().toRoochAddress()
      const authorizer = new PrivateKeyAuth(kp)

      const account = new Account(mockProvider, roochAddress, authorizer)
      expect(account).toBeDefined()

      account.runFunction('0x123::counter::increase', [], [], {
        maxGasAmount: 1000000,
      })
    })
  })
})
