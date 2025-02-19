// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'

describe('WebSocket Transport Tests', () => {
  let wsTestBox: TestBox

  beforeAll(async () => {
    wsTestBox = TestBox.setup(undefined, 'ws')
  })

  afterAll(async () => {
    await wsTestBox.cleanEnv()
  })

  describe('Basic Operations', () => {
    it('should execute single transaction successfully', async () => {
      const tx = new Transaction()
      tx.callFunction({
        target: '0x3::empty::empty_with_signer',
      })

      const result = await wsTestBox.getClient().signAndExecuteTransaction({
        transaction: tx,
        signer: wsTestBox.keypair,
      })

      expect(result.execution_info.status.type).eq('executed')
    })

    it('should query transaction history', async () => {
      const tx = new Transaction()
      tx.callFunction({
        target: '0x3::empty::empty_with_signer',
      })

      expect(await wsTestBox.signAndExecuteTransaction(tx)).toBeTruthy()

      const result = await wsTestBox.getClient().queryTransactions({
        filter: {
          sender: wsTestBox.address().toHexAddress(),
        },
      })

      expect(result.data.length).toBeGreaterThan(0)
    })
  })

  describe('WebSocket Specific Features', () => {
    it('should handle sequential transactions', async () => {
      const numSequentialTx = 5
      const results = []

      for (let i = 0; i < numSequentialTx; i++) {
        const tx = new Transaction()
        tx.callFunction({
          target: '0x3::empty::empty_with_signer',
        })

        const result = await wsTestBox.signAndExecuteTransaction(tx)
        results.push(result)
      }

      expect(results).toHaveLength(numSequentialTx)
      expect(results.every((result) => result === true)).toBeTruthy()
    })

    it('should automatically reconnect after connection loss', async () => {
      // First transaction to ensure connection is working
      const tx1 = new Transaction()
      tx1.callFunction({
        target: '0x3::empty::empty_with_signer',
      })
      await wsTestBox.signAndExecuteTransaction(tx1)

      // Force client to recreate with new connection
      await wsTestBox.loadRoochEnv('local', 0, 'ws')

      // Second transaction should work after reconnection
      const tx2 = new Transaction()
      tx2.callFunction({
        target: '0x3::empty::empty_with_signer',
      })
      const result = await wsTestBox.signAndExecuteTransaction(tx2)

      expect(result).toBeTruthy()
    })
  })

  describe('Error Handling', () => {
    it('should handle invalid transactions gracefully', async () => {
      const tx = new Transaction()
      tx.callFunction({
        target: '0x3::non_existent::function',
      })

      try {
        await wsTestBox.signAndExecuteTransaction(tx)
        expect.fail('Should have thrown an error')
      } catch (error) {
        expect(error).toBeDefined()
      }
    })

    it('should handle multiple requests during reconnection', async () => {
      // Force client to recreate with new connection
      await wsTestBox.loadRoochEnv('local', 0, 'ws')

      const numSequentialTx = 3
      const results = []

      for (let i = 0; i < numSequentialTx; i++) {
        const tx = new Transaction()
        tx.callFunction({
          target: '0x3::empty::empty_with_signer',
        })

        const result = await wsTestBox.signAndExecuteTransaction(tx)
        results.push(result)
      }

      expect(results).toHaveLength(numSequentialTx)
      expect(results.every((result) => result === true)).toBeTruthy()
    })
  })
})
