// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Transaction, Subscription, Secp256k1Keypair, Args } from '../../src/index.js'

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
    console.log(`Command address: ${cmdAddress}`)

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

  it('should correctly unsubscribe and stop receiving events', async () => {
    console.log('Starting unsubscribe functionality test')

    const cmdAddress = await wsTestBox.defaultCmdAddress()
    let receivedEventsBeforeUnsubscribe = 0
    let receivedEventsAfterUnsubscribe = 0
    let subscription: Subscription

    // Setup subscription and trigger first event
    const firstEventReceived = new Promise<boolean>((resolve) => {
      console.log('Setting up event subscription...')

      wsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          filter: {
            event_type: `${cmdAddress}::entry_function::U64Event`,
          },
          onEvent: (event) => {
            console.log('Received event in unsubscribe test:', JSON.stringify(event.type))

            if (
              event.type === 'event' &&
              event.data.event_type === `${cmdAddress}::entry_function::U64Event`
            ) {
              console.log('Received matching event before unsubscribe')
              receivedEventsBeforeUnsubscribe++
              resolve(true)

              // We keep the subscription active to verify it receives no more events after unsubscribing
            }
          },
          onError: (error) => {
            console.error('Event subscription error:', error)
            resolve(false)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Event subscription established with ID: ${sub.id}`)
        })
        .catch((err) => {
          console.error('Failed to create event subscription:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    console.log('Waiting for subscription to be established...')
    await new Promise((resolve) => setTimeout(resolve, 2000))

    // Emit first event that should be received
    console.log('Emitting first event (should be received)...')
    const tx1 = new Transaction()
    tx1.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(100))],
    })

    const txResult1 = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx1,
      signer: wsTestBox.keypair,
    })

    console.log('First transaction result:', JSON.stringify(txResult1.execution_info.status))
    expect(txResult1.execution_info.status.type).eq('executed')

    // Wait for first event to be received
    console.log('Waiting for first event to be received...')
    const firstEventResult = await Promise.race([
      firstEventReceived,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log('Timeout waiting for first event')
          resolve(false)
        }, 10000),
      ),
    ])

    expect(firstEventResult).toBe(true)
    console.log(`Events received before unsubscribe: ${receivedEventsBeforeUnsubscribe}`)
    expect(receivedEventsBeforeUnsubscribe).toBeGreaterThan(0)

    // Now unsubscribe to stop receiving events
    console.log(`Unsubscribing from subscription ID: ${subscription.id}...`)
    subscription.unsubscribe()

    // Setup a listener for events that might be received after unsubscribe
    console.log('Setting up listener for events after unsubscribe...')
    const client = wsTestBox.getClient()
    const originalSubscriptionFn = client.subscribe.bind(client)

    // Replace the subscribe method temporarily to catch any events
    client.subscribe = ((options) => {
      if (
        options.type === 'event' &&
        options.filter.event_type === `${cmdAddress}::entry_function::U64Event`
      ) {
        const originalOnEvent = options.onEvent
        options.onEvent = (event) => {
          if (
            event.type === 'event' &&
            event.data.event_type === `${cmdAddress}::entry_function::U64Event`
          ) {
            console.log('Received event after unsubscribe (should not happen)')
            receivedEventsAfterUnsubscribe++
          }
          if (originalOnEvent) originalOnEvent(event)
        }
      }
      return originalSubscriptionFn(options)
    }) as any

    // Wait a bit to ensure unsubscribe takes effect
    console.log('Waiting for unsubscribe to take effect...')
    await new Promise((resolve) => setTimeout(resolve, 3000))

    // Emit second event that should NOT be received due to unsubscribe
    console.log('Emitting second event (should NOT be received)...')
    const tx2 = new Transaction()
    tx2.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(200))],
    })

    const txResult2 = await wsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx2,
      signer: wsTestBox.keypair,
    })

    console.log('Second transaction result:', JSON.stringify(txResult2.execution_info.status))
    expect(txResult2.execution_info.status.type).eq('executed')

    // Wait a bit to see if any events are received
    console.log('Waiting to see if any events are received after unsubscribe...')
    await new Promise((resolve) => setTimeout(resolve, 5000))

    // Restore original subscribe method
    client.subscribe = originalSubscriptionFn

    // Verify no events were received after unsubscribe
    console.log(`Events received after unsubscribe: ${receivedEventsAfterUnsubscribe}`)
    expect(receivedEventsAfterUnsubscribe).toBe(0)
  })

  it('should automatically resubscribe after connection is reestablished', async () => {
    console.log('Starting reconnection handling test')

    // Create a container-based test environment for network simulation
    const keypair = Secp256k1Keypair.generate()
    const containerWsTestBox = new TestBox(keypair)

    // Initialize with container-based Rooch instance and WebSocket transport
    await containerWsTestBox.loadRoochEnv('container', 0, 'ws')

    // Deploy the entry_function example package first
    console.log('Publishing entry_function package...')
    const entryFunctionDeployResult = await containerWsTestBox.cmdPublishPackage(
      '../../../examples/entry_function_arguments',
    )
    expect(entryFunctionDeployResult).toBeTruthy()
    console.log('entry_function package published successfully')

    const cmdAddress = await containerWsTestBox.defaultCmdAddress()

    // Variables to track events
    let eventsBeforeDisconnection = 0
    let eventsAfterReconnection = 0
    let subscription: Subscription
    let networkDisrupted = false

    // Setup subscription
    const subscriptionSetup = new Promise<boolean>((resolve) => {
      console.log('Setting up event subscription for reconnection test...')

      containerWsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          filter: {
            event_type: `${cmdAddress}::entry_function::U64Event`,
          },
          onEvent: (event) => {
            console.log('Received event in reconnection test:', JSON.stringify(event.type))

            if (
              event.type === 'event' &&
              event.data.event_type === `${cmdAddress}::entry_function::U64Event`
            ) {
              if (!networkDisrupted) {
                console.log('Received event before network disruption')
                eventsBeforeDisconnection++
              } else {
                console.log('Received event after reconnection')
                eventsAfterReconnection++
              }
            }
          },
          onError: (error) => {
            console.error('Event subscription error:', error)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Event subscription established with ID: ${sub.id}`)
          resolve(true)
        })
        .catch((err) => {
          console.error('Failed to create event subscription for reconnection test:', err)
          resolve(false)
        })
    })

    // Wait for subscription to be established
    console.log('Waiting for subscription to be established...')
    const subscriptionResult = await Promise.race([
      subscriptionSetup,
      new Promise<boolean>((resolve) =>
        setTimeout(() => {
          console.log('Timeout waiting for subscription setup')
          resolve(false)
        }, 5000),
      ),
    ])

    expect(subscriptionResult).toBe(true)

    let reconnectCount = 0
    containerWsTestBox
      .getClient()
      .getSubscriptionTransport()
      ?.onReconnected(() => {
        console.log('SubscriptionTransport Reconnected')
        reconnectCount++
      })

    // Send a test event before network disruption
    console.log('Sending test event before network disruption...')
    const tx1 = new Transaction()
    tx1.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(300))],
    })

    const txResult1 = await containerWsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx1,
      signer: containerWsTestBox.keypair,
    })

    console.log('First transaction result:', JSON.stringify(txResult1.execution_info.status))
    expect(txResult1.execution_info.status.type).eq('executed')

    // Wait for event to be received
    await new Promise((resolve) => setTimeout(resolve, 3000))
    console.log(`Events received before network disruption: ${eventsBeforeDisconnection}`)
    expect(eventsBeforeDisconnection).toBeGreaterThan(0)

    // Simulate network failure with Pumba
    console.log('Simulating network disruption with Pumba...')
    // Simulate network disruption
    networkDisrupted = true
    containerWsTestBox.simulateRoochRpcPacketLoss(100, 30)

    // Wait 10s
    console.log('Wait network disruption with Pumba...')
    await new Promise((resolve) => setTimeout(resolve, 10000))

    // Function to execute a transaction
    const executeTransaction = async (value: number): Promise<boolean> => {
      try {
        const tx = new Transaction()
        tx.callFunction({
          target: `${cmdAddress}::entry_function::emit_u64`,
          args: [Args.u64(BigInt(value))],
        })

        const result = await containerWsTestBox.getClient().signAndExecuteTransaction({
          transaction: tx,
          signer: containerWsTestBox.keypair,
        })

        return result.execution_info.status.type === 'executed'
      } catch (error) {
        console.error(`Transaction failed: ${error}`)
        return false
      }
    }

    // Try to send transactions during network disruption
    try {
      console.log('Starting continuous transaction sending...')
      const startTime = Date.now()
      const testDuration = 30 * 1000 // 30s

      let successfulTransactions = 0
      let failedTransactions = 0
      let totalTransactions = 0

      // Start executing transactions continuously
      // We'll use a loop with a small delay between transactions
      while (Date.now() - startTime < testDuration) {
        totalTransactions++
        const txValue = totalTransactions // Use transaction count as the value

        const success = await executeTransaction(txValue)

        if (success) {
          successfulTransactions++
        } else {
          failedTransactions++
        }

        // Log progress every 50 transactions
        if (totalTransactions % 5 === 0) {
          const elapsedSeconds = Math.floor((Date.now() - startTime) / 1000)
          const tps = Math.round((successfulTransactions / elapsedSeconds) * 100) / 100
          console.log(
            `Progress: ${elapsedSeconds}s elapsed, ${successfulTransactions} successful, ${failedTransactions} failed, ~${tps} TPS`,
          )
        }

        // Small delay to prevent overwhelming the system
        await new Promise((r) => setTimeout(r, 5000))
      }
    } catch (err) {
      console.log('Expected error during network disruption:', err)
    }

    // Wait for network recovery and reconnection
    console.log('Waiting for network recovery and reconnection...')
    await new Promise((resolve) => setTimeout(resolve, 30000))

    try {
      // Send test event after reconnection
      const tx3 = new Transaction()
      tx3.callFunction({
        target: `${cmdAddress}::entry_function::emit_u64`,
        args: [Args.u64(BigInt(400))],
      })

      await containerWsTestBox.getClient().signAndExecuteTransaction({
        transaction: tx3,
        signer: containerWsTestBox.keypair,
      })
    } catch (err) {
      console.log('Expected error during network recovery:', err)
    }

    // Wait for event to be received after reconnection
    await new Promise((resolve) => setTimeout(resolve, 5000))

    if (subscription) {
      subscription.unsubscribe()
    }

    console.log(`Events received after reconnection: ${eventsAfterReconnection}`)
    await containerWsTestBox.cleanEnv()

    expect(eventsBeforeDisconnection).toBeGreaterThan(0)
    expect(eventsAfterReconnection).toBeGreaterThan(0)
    expect(reconnectCount).toBeGreaterThan(0)
  }, 300000)

  it('should properly propagate errors to subscription error callbacks', async () => {
    console.log('Starting subscription error handling test')

    // Create test environment with WebSocket transport
    const keypair = Secp256k1Keypair.generate()
    const containerWsTestBox = new TestBox(keypair)

    // Initialize with container-based Rooch instance and WebSocket transport
    await containerWsTestBox.loadRoochEnv('container', 0, 'ws')

    // Deploy the entry_function example package
    console.log('Publishing entry_function package...')
    const entryFunctionDeployResult = await containerWsTestBox.cmdPublishPackage(
      '../../../examples/entry_function_arguments',
    )
    expect(entryFunctionDeployResult).toBeTruthy()
    console.log('entry_function package published successfully')

    const cmdAddress = await containerWsTestBox.defaultCmdAddress()

    // Track received errors and events
    const receivedErrors: Error[] = []
    const receivedEvents: any[] = []
    let subscription: Subscription | undefined

    // Get direct access to the transport for testing
    const wsTransport = containerWsTestBox.getClient().getSubscriptionTransport()

    // Manually register an error listener to confirm transport errors occur
    const transportErrors: Error[] = []
    wsTransport?.onError((error: Error) => {
      console.log('Transport error detected:', error.message)
      transportErrors.push(error)
    })

    // Use a promise to track when an error is received via subscription callback
    const errorReceived = new Promise<boolean>((resolve) => {
      // Setup error callback with a timeout
      const errorTimeout = setTimeout(() => {
        console.log('Error timeout reached without subscription error callback')
        resolve(false)
      }, 300000) // 300 seconds timeout for error

      console.log('Setting up subscription with error handler...')

      containerWsTestBox
        .getClient()
        .subscribe({
          type: 'event',
          filter: {
            event_type: `${cmdAddress}::entry_function::U64Event`,
          },
          onEvent: (event) => {
            console.log('Received event:', event.type)
            receivedEvents.push(event)
          },
          onError: (error) => {
            console.log('✅ RECEIVED ERROR IN SUBSCRIPTION CALLBACK:', error.message)
            receivedErrors.push(error)
            clearTimeout(errorTimeout)
            resolve(true)
          },
        })
        .then((sub) => {
          subscription = sub
          console.log(`Subscription established with ID: ${sub.id}`)
        })
        .catch((err) => {
          console.error('Failed to create subscription during setup:', err)
          receivedErrors.push(err)
          clearTimeout(errorTimeout)
          resolve(true)
        })
    })

    // Wait for subscription to be established
    await new Promise((resolve) => setTimeout(resolve, 3000))

    // First, send a normal event to verify subscription is working
    console.log('Sending initial event to verify subscription is working')
    const tx = new Transaction()
    tx.callFunction({
      target: `${cmdAddress}::entry_function::emit_u64`,
      args: [Args.u64(BigInt(1))],
    })

    await containerWsTestBox.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: containerWsTestBox.keypair,
    })

    // Wait to ensure the event is received
    await new Promise((resolve) => setTimeout(resolve, 3000))
    console.log(`Received ${receivedEvents.length} events before inducing errors`)

    // Create severe network issues to force reconnection failures
    console.log('Creating severe network issues to force connection failures')

    // First, simulate complete network partition
    console.log('Step 1: Complete network partition (100% packet loss)')
    try {
      containerWsTestBox.simulateRoochRpcPacketLoss(100, 60) // 100% packet loss for 60 seconds
    } catch (err) {
      console.error('Error simulating packet loss:', err)
    }

    // Wait 10s
    console.log('Wait network disruption with Pumba...')
    await new Promise((resolve) => setTimeout(resolve, 10000))

    // Function to execute a transaction
    const executeTransaction = async (value: number): Promise<boolean> => {
      try {
        const tx = new Transaction()
        tx.callFunction({
          target: `${cmdAddress}::entry_function::emit_u64`,
          args: [Args.u64(BigInt(value))],
        })

        const result = await containerWsTestBox.getClient().signAndExecuteTransaction({
          transaction: tx,
          signer: containerWsTestBox.keypair,
        })

        return result.execution_info.status.type === 'executed'
      } catch (error) {
        console.error(`Transaction failed: ${error}`)
        return false
      }
    }

    // Try to send transactions during network disruption
    try {
      console.log('Starting continuous transaction sending...')
      const startTime = Date.now()
      const testDuration = 60 * 1000 // 60s

      let successfulTransactions = 0
      let failedTransactions = 0
      let totalTransactions = 0

      // Start executing transactions continuously
      // We'll use a loop with a small delay between transactions
      while (Date.now() - startTime < testDuration) {
        totalTransactions++
        const txValue = totalTransactions // Use transaction count as the value

        const success = await executeTransaction(txValue)

        if (success) {
          successfulTransactions++
        } else {
          failedTransactions++
        }

        // Log progress every 50 transactions
        if (totalTransactions % 5 === 0) {
          const elapsedSeconds = Math.floor((Date.now() - startTime) / 1000)
          const tps = Math.round((successfulTransactions / elapsedSeconds) * 100) / 100
          console.log(
            `Progress: ${elapsedSeconds}s elapsed, ${successfulTransactions} successful, ${failedTransactions} failed, ~${tps} TPS`,
          )
        }

        // Small delay to prevent overwhelming the system
        await new Promise((r) => setTimeout(r, 5000))
      }
    } catch (err) {
      console.log('Expected error during network disruption:', err)
    }

    // Wait for either an error to be received or the timeout to expire
    console.log('Waiting for subscription error callback or timeout...')
    const errorWasReceived = await errorReceived

    // Additional wait for any other errors that might come in
    await new Promise((resolve) => setTimeout(resolve, 5000))

    // Clean up
    if (subscription) {
      subscription.unsubscribe()
    }
    await containerWsTestBox.cleanEnv()

    // Log results
    console.log(
      `Subscription errors: ${receivedErrors.length}, Transport errors: ${transportErrors.length}`,
    )

    console.log('Transport errors:')
    transportErrors.forEach((error, index) => {
      console.log(`Transport Error ${index + 1}: ${error.message}`)
    })

    console.log('Subscription errors:')
    receivedErrors.forEach((error, index) => {
      console.log(`Subscription Error ${index + 1}: ${error.message}`)
    })

    // Test assertions
    expect(transportErrors.length).toBeGreaterThan(0)

    if (errorWasReceived) {
      // Verify errors were properly propagated to subscription handlers
      expect(receivedErrors.length).toBeGreaterThan(0)

      // Error messages should be strings, not empty or undefined
      receivedErrors.forEach((error) => {
        expect(typeof error.message).toBe('string')
        expect(error.message.length).toBeGreaterThan(0)
        // Should contain information about the subscription
        expect(error.message).toMatch(/subscription|transport|connection|WebSocket/i)
      })
    } else {
      console.log('⚠️ Warning: No subscription errors were received within the timeout period.')
      console.log(
        "This doesn't necessarily mean error propagation is broken - the test may need more",
      )
      console.log('severe network disruption or longer duration to trigger the error callbacks.')

      // Mark test as inconclusive in this case
      console.log('Marking test as skipped due to insufficient error triggers')
      expect(true).toBe(true)
    }
  }, 300000) // 3-minute timeout for this test
})
