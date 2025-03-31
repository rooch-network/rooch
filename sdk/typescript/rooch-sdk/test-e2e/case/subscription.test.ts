// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction } from '../../src/transactions/index.js'
import { Subscription } from '../../src/client/subscriptionTransportInterface.js'
import { Args } from '../../src/bcs/index.js'

describe('RoochClient Subscription Tests', () => {
  let wsTestBox: TestBox

  beforeAll(async () => {
    wsTestBox = TestBox.setup(undefined, 'ws')
    console.log('Setting up test environment with WebSocket transport')

    // Deploy the event example package
    console.log('Publishing event package...')
    const result = await wsTestBox.cmdPublishPackage('../../../examples/event')
    expect(result).toBeTruthy()
    console.log('Event package published successfully')
  })

  afterAll(async () => {
    console.log('Cleaning up test environment')
    await wsTestBox.cleanEnv()
  })

  it('should subscribe to events and receive event notifications', async () => {
    console.log('Starting event subscription test')

    let receivedEvents = []

    // Create a promise that will resolve when we receive the expected event
    const eventReceived = new Promise<boolean>((resolve) => {
      let subscription: Subscription

      console.log('Setting up subscription...')
      wsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          onEvent: (event) => {
            console.log('Received event:', JSON.stringify(event))
            receivedEvents.push(event)

            // Check if this is a WithdrawEvent from our deployed contract
            if (
              event.type === 'event' &&
              event.data.type_tag.includes('event_test::WithdrawEvent')
            ) {
              console.log('Found matching event, resolving promise')
              subscription?.unsubscribe()
              resolve(true)
            }
          },
          onError: (error) => {
            console.error('Subscription error:', error)
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Subscription established with ID: ${sub.id}`)
        })
        .catch((err) => {
          console.error('Failed to create subscription:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    console.log('Waiting for subscription to be established...')
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // Get the address for calling the function
    const cmdAddress = await wsTestBox.defaultCmdAddress()
    console.log(`Using command address: ${cmdAddress}`)

    // Execute a transaction that emits an event
    const tx = new Transaction()
    const target = `${cmdAddress}::event_test::emit_event`
    console.log(`Calling function: ${target}`)

    tx.callFunction({
      target: target,
      args: [Args.u64(BigInt(10))],
    })

    console.log('Executing transaction...')
    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    console.log('Transaction result:', JSON.stringify(txResult))
    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for the event notification with a timeout
    console.log('Waiting for event notification (timeout: 15s)...')
    const result = await Promise.race([
      eventReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log('Timeout waiting for event. Events received so far:', receivedEvents.length)
          resolve(false)
        }, 15000),
      ),
    ])

    // Log all received events for debugging
    console.log(`Total events received: ${receivedEvents.length}`)
    if (receivedEvents.length > 0) {
      console.log('Event types received:')
      receivedEvents.forEach((evt, i) => {
        if (evt.type === 'event') {
          console.log(`Event ${i}: ${evt.data.type_tag}`)
        } else {
          console.log(`Event ${i}: ${evt.type}`)
        }
      })
    }

    expect(result).toBe(true)
  })
})
