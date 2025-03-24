// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { TestBox } from './setup'
import { IndexerEventView, TransactionWithInfoView } from '../src/types'

describe('WebSocket Transport Tests', () => {
  const testBox = TestBox.setup(undefined, 'ws')

  it('should subscribe to events', async () => {
    const client = testBox.getClient()

    const eventPromise = new Promise<IndexerEventView>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Event subscription timed out'))
      }, 5000)

      const unsubscribe = client.subscribeEvents({
        filter: { type: 'All' },
        onMessage: (event) => {
          clearTimeout(timeout)
          unsubscribe()
          resolve(event)
        },
        onError: (error) => {
          clearTimeout(timeout)
          unsubscribe()
          reject(error)
        },
      })
    })

    const event = await eventPromise
    expect(event).toBeDefined()
  })

  it('should subscribe to transactions', async () => {
    const client = testBox.getClient()

    const txPromise = new Promise<TransactionWithInfoView>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Transaction subscription timed out'))
      }, 5000)

      const unsubscribe = client.subscribeTransactions({
        filter: { type: 'All' },
        onMessage: (tx) => {
          clearTimeout(timeout)
          unsubscribe()
          resolve(tx)
        },
        onError: (error) => {
          clearTimeout(timeout)
          unsubscribe()
          reject(error)
        },
      })
    })

    const tx = await txPromise
    expect(tx).toBeDefined()
  })

  it('should handle multiple subscriptions', async () => {
    const client = testBox.getClient()
    const subscriptions: Array<() => void> = []
    const receivedEvents: IndexerEventView[] = []

    const multiSubPromise = new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Multiple subscriptions timed out'))
      }, 10000)

      for (let i = 0; i < 3; i++) {
        const unsubscribe = client.subscribeEvents({
          filter: { type: 'All' },
          onMessage: (event) => {
            receivedEvents.push(event)
            if (receivedEvents.length === 3) {
              clearTimeout(timeout)
              subscriptions.forEach((unsub) => unsub())
              resolve()
            }
          },
          onError: (error) => {
            clearTimeout(timeout)
            subscriptions.forEach((unsub) => unsub())
            reject(error)
          },
        })
        subscriptions.push(unsubscribe)
      }
    })

    await multiSubPromise
    expect(receivedEvents.length).toBe(3)
  })

  it('should handle subscription errors', async () => {
    const client = testBox.getClient()

    const errorPromise = new Promise<Error>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Error subscription timed out'))
      }, 5000)

      const unsubscribe = client.subscribeEvents({
        filter: { type: 'Invalid' }, // Invalid filter type to trigger error
        onMessage: () => {
          clearTimeout(timeout)
          unsubscribe()
          reject(new Error('Should not receive message for invalid subscription'))
        },
        onError: (error) => {
          clearTimeout(timeout)
          unsubscribe()
          resolve(error)
        },
      })
    })

    const error = await errorPromise
    expect(error).toBeInstanceOf(Error)
  })
}) 