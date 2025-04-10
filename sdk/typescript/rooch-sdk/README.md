# Rooch TypeScript SDK

This is the Rooch TypeScript SDK built on the Rooch [JSON RPC API](https://github.com/rooch-network/rooch/blob/main/crates/rooch-open-rpc-spec/schemas/openrpc.json). It provides utility classes and functions for applications to sign transactions and interact with the Rooch network.

WARNING: Note that we are still iterating on the RPC and SDK API before TestNet, therefore please expect frequent breaking changes in the short-term. We expect the API to stabilize after the upcoming TestNet launch.

## Installation

```shell
npm i @roochnetwork/rooch-sdk
```

## Connecting to Rooch Network

The SDK supports both HTTP and WebSocket connections. You can choose the appropriate transport based on your needs:

### HTTP Connection (Default)

```typescript
import { RoochClient, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

// create a client connected to testnet
const client = new RoochClient({
  url: getRoochNodeUrl('testnet'),
})

// get balances
await client.getBalances({
  owner: '',
})
```

### WebSocket Connection

```typescript
import { RoochClient, RoochWebSocketTransport, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

// Create WebSocket transport with custom options
const wsTransport = new RoochWebSocketTransport({
  url: getRoochNodeUrl('testnet'),
  reconnectDelay: 1000, // Delay between reconnection attempts (default: 1000ms)
  maxReconnectAttempts: 5, // Maximum number of reconnection attempts (default: 5)
  requestTimeout: 30000, // Request timeout (default: 30000ms)
  connectionReadyTimeout: 5000, // Connection ready timeout (default: 5000ms)
})

// Create client with WebSocket transport
const client = new RoochClient({
  transport: wsTransport,
})

// Use client as normal
await client.getBalances({
  owner: '',
})

// Clean up resources when done
client.destroy()
```

The WebSocket transport provides additional features:

- Automatic reconnection on connection loss
- Configurable timeouts and retry attempts
- Connection state management
- Resource cleanup

You can customize the WebSocket behavior through the following options:

- `url`: WebSocket endpoint URL (required)
- `reconnectDelay`: Delay between reconnection attempts in milliseconds
- `maxReconnectAttempts`: Maximum number of reconnection attempts
- `requestTimeout`: Timeout for individual requests
- `connectionReadyTimeout`: Timeout for waiting for connection to be ready
- `protocols`: WebSocket sub-protocols (optional)

## Writing APIs

Session Account

```typescript
import { RoochClient, Secp256k1Keypair, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

const client = new RoochClient({
  url: getRoochNodeUrl('testnet'),
})

const kp = Secp256k1Keypair.generate()

const session = await client.createSession({
  sessionArgs: {
    appName: 'your app name',
    appUrl: 'your app url',
    scopes: ['0x3::empty::empty_with_signer'],
  },
  signer: kp,
})
```

### Move Call

```typescript
import { RoochClient, getRoochNodeUrl, Transaction } from '@roochnetwork/rooch-sdk'

const client = new RoochClient({
  url: getRoochNodeUrl('testnet'),
})

const tx = new Transaction()
tx.callFunction({
  target: '0x3::empty::empty_with_signer',
  maxGas: 100000000, // 1RGas, DEFAULT_GAS 50000000 = 0.5RGas
})

const result = await client.signAndExecuteTransaction({
  transaction: tx,
  signer: session,
})
```

## Reading APIs

### Move view

```typescript
import { RoochClient, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

const client = new RoochClient({
  url: getRoochNodeUrl('devnet'),
})

const result = await client.executeViewFunction(
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::counter::view',
)
```

## Subscriptions

The SDK supports real-time subscriptions to events and transactions using WebSocket connections. Subscriptions allow your application to receive updates in real-time without polling.

### Setting up a Subscription

To use subscriptions, you must initialize your `RoochClient` with a WebSocket transport:

```typescript
import { RoochClient, RoochWebSocketTransport, getRoochNodeUrl } from '@roochnetwork/rooch-sdk'

// Create WebSocket transport
const wsTransport = new RoochWebSocketTransport({
  url: getRoochNodeUrl('testnet'),
})

// Create client with WebSocket transport
const client = new RoochClient({
  transport: wsTransport,
  subscriptionTransport: wsTransport, // Use the same transport for subscriptions
})

// Now you can use subscriptions
```

### Subscribing to Events

```typescript
// Subscribe to all events
const eventSubscription = await client.subscribe({
  type: 'event',
  onEvent: (event) => {
    console.log('Received event:', event.data)
  },
  onError: (error) => {
    console.error('Subscription error:', error)
  },
})

// Subscribe to events with a filter
const filteredEventSubscription = await client.subscribe({
  type: 'event',
  filter: {
    sender: '0x123...', // Filter events by sender
  },
  onEvent: (event) => {
    console.log('Received filtered event:', event.data)
  },
})

// Later, unsubscribe when no longer needed
client.unsubscribe(eventSubscription.id)
```

### Subscribing to Transactions

```typescript
// Subscribe to all transactions
const txSubscription = await client.subscribe({
  type: 'transaction',
  onEvent: (event) => {
    console.log('Received transaction:', event.data)
  },
  onError: (error) => {
    console.error('Subscription error:', error)
  },
})

// Subscribe to transactions with a filter
const filteredTxSubscription = await client.subscribe({
  type: 'transaction',
  filter: {
    sender: '0x123...', // Filter transactions by sender
  },
  onEvent: (event) => {
    console.log('Received filtered transaction:', event.data)
  },
})

// Later, unsubscribe when no longer needed
client.unsubscribe(txSubscription.id)
```

### Subscription Events

The

onEvent

callback receives an object with:

-

type

: Either 'event' or 'transaction'

-

data

: The event or transaction data

### Handling Connection Issues

The WebSocket transport automatically handles reconnection on connection loss. You can configure its behavior using the options described in the WebSocket Connection section above.

When reconnection happens, subscriptions are automatically re-established.

### Cleaning Up

Always clean up resources when done:

```typescript
// Unsubscribe from all subscriptions
client.unsubscribe(subscription1.id)
client.unsubscribe(subscription2.id)

// Destroy the client to free resources
client.destroy()
```

## Building Locally

To get started you need to install [pnpm](https://pnpm.io/), then run the following command:

```bash
# Install all dependencies
$ pnpm install
# Run the build for the TypeScript SDK
$ pnpm build
```

> All `pnpm` commands are intended to be run in the root of the Rooch repo. You can also run them within the `sdk/typescript` directory, and remove change `pnpm sdk` to just `pnpm` when running commands.

## Type Doc

For the latest docs for the `main` branch, run `pnpm doc` and open the [doc/index.html](doc/index.html) in your browser.

## Testing

To run tests

```
pnpm test
```

## check compatibility

```
pnpm gen
pnpm test
```
