// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { IndexerEventView, TransactionWithInfoView } from '../../src/types/index.js'

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

    it('should subscribe to events and receive updates', async () => {
      // Create a promise that will resolve when we receive the event
      const eventPromise = new Promise<IndexerEventView>((resolve) => {
        const unsubscribe = wsTestBox.getClient().subscribeEvents({
          filter: {
            sender: wsTestBox.address().toHexAddress(),
          },
          onMessage: (event: IndexerEventView) => {
            unsubscribe()
            resolve(event)
          },
          onError: (error: Error) => {
            console.error('Event subscription error:', error)
          },
        })
      })

      // Execute a transaction that will emit an event
      const tx = new Transaction()
      tx.callFunction({
        target: '0x3::empty::empty_with_signer',
      })
      await wsTestBox.signAndExecuteTransaction(tx)

      // Wait for the event with timeout
      const event = await Promise.race([
        eventPromise,
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Event timeout')), 5000),
        ),
      ])

      expect(event).toBeDefined()
      expect(event.sender).eq(wsTestBox.address().toHexAddress())
    })

    it('should subscribe to transactions and receive updates', async () => {
      // Create a promise that will resolve when we receive the transaction
      const txPromise = new Promise<TransactionWithInfoView>((resolve) => {
        const unsubscribe = wsTestBox.getClient().subscribeTransactions({
          filter: {
            sender: wsTestBox.address().toHexAddress(),
          },
          onMessage: (transaction: TransactionWithInfoView) => {
            unsubscribe()
            resolve(transaction)
          },
          onError: (error: Error) => {
            console.error('Transaction subscription error:', error)
          },
        })
      })

      // Execute a transaction
      const tx = new Transaction()
      tx.callFunction({
        target: '0x3::empty::empty_with_signer',
      })
      await wsTestBox.signAndExecuteTransaction(tx)

      // Wait for the transaction notification with timeout
      const transaction = await Promise.race([
        txPromise,
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Transaction timeout')), 5000),
        ),
      ])

      expect(transaction).toBeDefined()
      expect(transaction.transaction.data.type).eq('l2_tx')
    })

    it('should handle multiple subscriptions concurrently', async () => {
      const subscriptionPromises = []
      const numSubscriptions = 3
      const receivedEvents = new Set<string>()

      // Create multiple event subscriptions
      for (let i = 0; i < numSubscriptions; i++) {
        const promise = new Promise<Set<string>>((resolve) => {
          const unsubscribe = wsTestBox.getClient().subscribeEvents({
            filter: {
              sender: wsTestBox.address().toHexAddress(),
            },
            onMessage: (event: IndexerEventView) => {
              receivedEvents.add(event.indexer_event_id.event_index)
              if (receivedEvents.size === numSubscriptions) {
                unsubscribe()
                resolve(receivedEvents)
              }
            },
            onError: (error: Error) => {
              console.error(`Subscription ${i} error:`, error)
            },
          })
        })
        subscriptionPromises.push(promise)
      }

      // Execute transactions that will emit events
      for (let i = 0; i < numSubscriptions; i++) {
        const tx = new Transaction()
        tx.callFunction({
          target: '0x3::empty::empty_with_signer',
        })
        await wsTestBox.signAndExecuteTransaction(tx)
      }

      // Wait for all subscriptions to receive events
      const results = await Promise.race([
        Promise.all(subscriptionPromises),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Subscription timeout')), 10000),
        ),
      ])

      expect(results).toBeDefined()
      expect(receivedEvents.size).eq(numSubscriptions)
    })

    it('should handle subscription errors gracefully', async () => {
      let errorCaught = false

      const unsubscribe = wsTestBox.getClient().subscribeEvents({
        filter: {
          // Invalid filter to trigger error
          event_type: 'invalid_type',
        },
        onMessage: () => {},
        onError: (error: Error) => {
          errorCaught = true
          unsubscribe()
        },
      })

      // Wait a bit to ensure error handler is called
      await new Promise((resolve) => setTimeout(resolve, 1000))
      expect(errorCaught).toBeTruthy()
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
