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
    // Deploy the event example package
    const result = await wsTestBox.cmdPublishPackage('../../../examples/event')
    expect(result).toBeTruthy()
  })

  afterAll(async () => {
    await wsTestBox.cleanEnv()
  })

  it('should subscribe to events and receive event notifications', async () => {
    let receivedEvents = new Array<any>()

    // Create a promise that will resolve when we receive the expected event
    const eventReceived = new Promise<boolean>((resolve) => {
      let subscription: Subscription

      wsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          onEvent: (event) => {
            receivedEvents.push(event)

            // Check if this is a WithdrawEvent from our deployed contract
            if (
              event.type === 'event' &&
              event.data.event_type.includes('event_test::WithdrawEvent')
            ) {
              subscription?.unsubscribe()
              resolve(true)
            }
          },
          onError: (error) => {
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
        })
        .catch((err) => {
          resolve(false)
        })
    })

    // Wait for subscription to be established
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // Get the address for calling the function
    const cmdAddress = await wsTestBox.defaultCmdAddress()

    // Execute a transaction that emits an event
    const tx = new Transaction()
    const target = `${cmdAddress}::event_test::emit_event`

    tx.callFunction({
      target: target,
      args: [Args.u64(BigInt(10))],
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for the event notification with a timeout
    const result = await Promise.race([
      eventReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          resolve(false)
        }, 15000),
      ),
    ])

    expect(result).toBe(true)
  })

  it('should subscribe to transactions and receive transaction notifications', async () => {
    let receivedTransactions = new Array<any>()

    // Create a promise that will resolve when we receive the expected transaction
    const txReceived = new Promise<boolean>((resolve) => {
      let subscription: Subscription
      const senderAddress = wsTestBox.address().toBech32Address()

      wsTestBox
        .getClient()
        .subscribe({
          type: 'transaction',
          onEvent: (event) => {
            receivedTransactions.push(event)

            // Check if this is a transaction from our sender
            if (event.type === 'transaction') {
              const txData = event.data.transaction.data
              if (txData.type === 'l2_tx' && txData.sender === senderAddress) {
                subscription.unsubscribe()
                resolve(true)
              }
            }
          },
          onError: (error) => {
            console.error('Subscription error:', error)
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
        })
        .catch((err) => {
          console.error('Failed to create transaction subscription:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // Execute a transaction
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for the transaction notification with a timeout
    const result = await Promise.race([
      txReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log(
            'Timeout waiting for transaction. Transactions received so far:',
            receivedTransactions.length,
          )
          resolve(false)
        }, 15000),
      ),
    ])

    expect(result).toBe(true)
  })

  it('should subscribe to events with filter and only receive matching events', async () => {
    console.log('Starting filtered event subscription test')

    // Deploy the entry_function example package first
    console.log('Publishing entry_function package...')
    const entryFunctionDeployResult = await wsTestBox.cmdPublishPackage(
      '../../../examples/entry_function_arguments',
    )
    expect(entryFunctionDeployResult).toBeTruthy()
    console.log('entry_function package published successfully')

    let receivedEvents = new Array<any>()
    const cmdAddress = await wsTestBox.defaultCmdAddress()

    // Create a promise that will resolve when we receive the expected event
    const eventReceived = new Promise<boolean>((resolve) => {
      let subscription: Subscription

      console.log('Setting up filtered event subscription...')

      // Create a filter for events with specific event type - we'll filter for U64Event
      const eventFilter = {
        event_type: `${cmdAddress}::entry_function::U64Event`,
      }

      console.log(`Using event filter: ${JSON.stringify(eventFilter)}`)

      wsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          filter: eventFilter,
          onEvent: (event) => {
            console.log('Received filtered event:', JSON.stringify(event.type))
            receivedEvents.push(event)

            if (
              event.type === 'event' &&
              event.data.event_type === `${cmdAddress}::entry_function::U64Event`
            ) {
              console.log('Received matching filtered event, resolving promise')
              subscription?.unsubscribe()
              resolve(true)
            }
          },
          onError: (error) => {
            console.error('Filtered subscription error:', error)
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Filtered event subscription established with ID: ${sub.id}`)
        })
        .catch((err) => {
          console.error('Failed to create filtered event subscription:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    console.log('Waiting for filtered subscription to be established...')
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // First emit a different event that should not be caught by our filter
    console.log('Emitting a different event (U8Event) that should not match filter...')
    const tx1 = new Transaction()
    tx1.callFunction({
      target: `${cmdAddress}::entry_function::emit_u8`,
      args: [Args.u8(123)],
    })

    await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx1,
      signer: wsTestBox.keypair,
    })

    // Wait a bit to ensure first event is processed
    await new Promise((resolve) => setTimeout(resolve, 3000))
    console.log(`Events received after first transaction: ${receivedEvents.length}`)

    // Now emit the event that should match our filter (U64Event)
    console.log('Now emitting event (U64Event) that should match filter...')
    const tx2 = new Transaction()
    tx2.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(20))],
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx2,
      signer: wsTestBox.keypair,
    })

    console.log(
      'Matching event transaction result:',
      JSON.stringify(txResult.execution_info.status),
    )
    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for the event notification with a timeout
    console.log('Waiting for filtered event notification (timeout: 15s)...')
    const result = await Promise.race([
      eventReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log('Timeout waiting for filtered event. Events received:', receivedEvents.length)
          resolve(false)
        }, 20000),
      ),
    ])

    // We should have received only the event matching our filter
    console.log(`Total filtered events received: ${receivedEvents.length}`)

    if (receivedEvents.length > 0) {
      console.log('Event types received:')
      receivedEvents.forEach((evt, i) => {
        if (evt.type === 'event') {
          console.log(`Event ${i}: ${evt.data.event_type}`)
        } else {
          console.log(`Event ${i}: ${evt.type}`)
        }
      })
    }

    // Only events matching our filter should be received
    const nonMatchingEvents = receivedEvents.filter(
      (evt) => evt.type === 'event' && !evt.data.event_type.includes('entry_function::U64Event'),
    )

    console.log(`Number of non-matching events received: ${nonMatchingEvents.length}`)
    expect(nonMatchingEvents.length).toBe(0)
    expect(result).toBe(true)
  })

  it('should subscribe to transactions with filter and only receive matching transactions', async () => {
    console.log('Starting filtered transaction subscription test')

    let receivedTransactions = new Array<any>()
    const senderAddress = wsTestBox.address().toBech32Address()

    // Create a promise that will resolve when we receive the expected transaction
    const txReceived = new Promise<boolean>((resolve) => {
      let subscription: Subscription

      console.log('Setting up filtered transaction subscription...')

      // Create a filter for transactions from a specific sender
      const txFilter = {
        sender: senderAddress,
      }

      console.log(`Using transaction filter by sender: ${JSON.stringify(txFilter)}`)

      wsTestBox
        .getClient()
        .subscribe({
          type: 'transaction',
          filter: txFilter,
          onEvent: (event) => {
            console.log('Received filtered transaction event:', JSON.stringify(event.type))
            receivedTransactions.push(event)

            if (event.type === 'transaction') {
              const txData = event.data.transaction.data
              console.log(
                `Transaction type: ${txData.type}, sender: ${txData.type === 'l2_tx' ? txData.sender : 'N/A'}`,
              )

              // Check that this is an l2_tx and from our sender
              if (txData.type === 'l2_tx' && txData.sender === senderAddress) {
                console.log('Received matching filtered transaction, resolving promise')
                subscription?.unsubscribe()
                resolve(true)
              }
            }
          },
          onError: (error) => {
            console.error('Filtered transaction subscription error:', error)
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Filtered transaction subscription established with ID: ${sub.id}`)
        })
        .catch((err) => {
          console.error('Failed to create filtered transaction subscription:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    console.log('Waiting for filtered transaction subscription to be established...')
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // Execute a transaction that should match our filter
    console.log('Executing transaction that should match our filter...')
    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    console.log('Transaction result:', JSON.stringify(txResult.execution_info.status))
    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for the transaction notification with a timeout
    console.log('Waiting for filtered transaction notification (timeout: 15s)...')
    const result = await Promise.race([
      txReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log(
            'Timeout waiting for filtered transaction. Transactions received:',
            receivedTransactions.length,
          )
          resolve(false)
        }, 15000),
      ),
    ])

    // We should have received transactions that match our filter
    console.log(`Total filtered transactions received: ${receivedTransactions.length}`)

    // Verify all received transactions are from our sender
    if (receivedTransactions.length > 0) {
      console.log('Checking all transactions are from our sender:')
      const allMatchSender = receivedTransactions.every((evt) => {
        if (evt.type === 'transaction') {
          const txData = evt.data.transaction.data
          if (txData.type === 'l2_tx') {
            console.log(`Transaction sender: ${txData.sender}, expected: ${senderAddress}`)
            return txData.sender === senderAddress
          }
        }
        return false
      })

      expect(allMatchSender).toBe(true)
    }

    expect(result).toBe(true)
  })

  it('should handle multiple subscriptions simultaneously', async () => {
    console.log('Starting multiple subscriptions test')

    const cmdAddress = await wsTestBox.defaultCmdAddress()
    const senderAddress = wsTestBox.address().toBech32Address()
    let eventReceived = false
    let transactionReceived = false

    // Initialize subscription variables with default values
    let eventSubscription: Subscription | null = null
    let transactionSubscription: Subscription | null = null

    // Promise that resolves when both event and transaction are received
    const bothReceived = new Promise<boolean>((resolve) => {
      const checkBothReceived = () => {
        if (eventReceived && transactionReceived) {
          console.log('Both event and transaction received, resolving promise')
          resolve(true)
        }
      }

      // Set up event subscription
      console.log('Setting up event subscription...')
      wsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          filter: {
            event_type: `${cmdAddress}::entry_function::U64Event`,
          },
          onEvent: (event) => {
            console.log('Received event:', JSON.stringify(event.type))

            if (
              event.type === 'event' &&
              event.data.event_type === `${cmdAddress}::entry_function::U64Event`
            ) {
              console.log('Received matching event in multiple subscription test')
              eventReceived = true
              checkBothReceived()
            }
          },
          onError: (error) => {
            console.error('Event subscription error:', error)
          },
        })
        .then((sub) => {
          eventSubscription = sub
          console.log(`Event subscription established with ID: ${sub.id}`)
        })

      // Set up transaction subscription
      console.log('Setting up transaction subscription...')
      wsTestBox
        .getClient()
        .subscribe({
          type: 'transaction',
          filter: {
            sender: senderAddress,
          },
          onEvent: (event) => {
            console.log('Received transaction event:', JSON.stringify(event.type))

            if (event.type === 'transaction') {
              const txData = event.data.transaction.data
              if (txData.type === 'l2_tx' && txData.sender === senderAddress) {
                console.log('Received matching transaction in multiple subscription test')
                transactionReceived = true
                checkBothReceived()
              }
            }
          },
          onError: (error) => {
            console.error('Transaction subscription error:', error)
          },
        })
        .then((sub) => {
          transactionSubscription = sub
          console.log(`Transaction subscription established with ID: ${sub.id}`)
        })
    })

    // Wait for subscriptions to be established
    console.log('Waiting for subscriptions to be established...')
    await new Promise((resolve) => setTimeout(resolve, 3000))

    // Execute a transaction that generates both a transaction and an event
    console.log('Executing transaction that will generate both a transaction and event...')
    const tx = new Transaction()
    tx.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(42))],
    })

    const txResult = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: wsTestBox.keypair,
    })

    console.log('Transaction result:', JSON.stringify(txResult.execution_info.status))
    expect(txResult.execution_info.status.type).eq('executed')

    // Wait for both the transaction and event to be received
    console.log('Waiting for both transaction and event notifications (timeout: 20s)...')
    const result = await Promise.race([
      bothReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log(
            `Timeout waiting for both notifications. Event received: ${eventReceived}, Transaction received: ${transactionReceived}`,
          )
          resolve(false)
        }, 20000),
      ),
    ])

    // Clean up subscriptions
    console.log('Cleaning up subscriptions...')
    if (eventSubscription) {
      eventSubscription.unsubscribe()
    }
    if (transactionSubscription) {
      transactionSubscription.unsubscribe()
    }

    console.log(`Test result: ${result}`)
    expect(eventReceived).toBe(true)
    expect(transactionReceived).toBe(true)
    expect(result).toBe(true)
  })
})
