# Rooch TypeScript SDK

This is the Rooch TypeScript SDK built on the Rooch [JSON RPC API](https://github.com/rooch-network/rooch/blob/main/crates/rooch-open-rpc-spec/schemas/openrpc.json). It provides utility classes and functions for applications to sign transactions and interact with the Rooch network.

WARNING: Note that we are still iterating on the RPC and SDK API before TestNet, therefore please expect frequent breaking changes in the short-term. We expect the API to stabilize after the upcoming TestNet launch.

## Installation

```shell
npm i @roochnetwork/sdk
```

## Connecting to Rooch Network

The JsonRpcProvider class provides a connection to the JSON-RPC Server and should be used for all read-only operations. The default URLs to connect with the RPC server are:

- local: http://127.0.0.1:500051
- DevNet: https://dev-seed.rooch.network::443

For local development, you can run cargo run server start to start a local network. Refer to this guide for more information.

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'

// create a provider connected to devnet
const provider = new JsonRpcProvider(DevChain)

// get transactions
await provider.getTransactionsByOrder(0, 10)
```

You can also construct your own in custom connections, with the URL for your own network

```typescript
import { JsonRpcProvider, Chain } from '@rooch/sdk'

// Definition custom chian
export const CustomChain = new Chain(CUSTOM_CHAIN_ID, 'CUSTOM_CHAIN_NAME', {
  url: CUSTOM_CHAIN_URL,
})

const provider = new JsonRpcProvider(CustomChain)

// get transactions
await provider.getTransactionsByOrder(0, 10)
```

## Writing APIs

Rooch Account

```typescript
import { JsonRpcProvider, DevChain, Account, Ed25519Keypair } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)
const pk = Ed25519Keypair.generate()
const authorizer = new PrivateKeyAuth(pk)

const keypairAccount = Account(provider, account.roochAddress, authorizer)
```

Session Account

```typescript
import { JsonRpcProvider, DevChain, Account, Ed25519Keypair } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)
const pk = Ed25519Keypair.generate()
const authorizer = new PrivateKeyAuth(pk)

const sessionAccount = new Account(provider, account.roochAddress, authorizer).createSessionAccount(
  scope,
  maxInactiveInterval,
  opts,
)
```

### Move Call

```typescript
import { JsonRpcProvider, DevChain, Account, Ed25519Keypair } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)
const pk = Ed25519Keypair.generate()
const authorizer = new PrivateKeyAuth(pk)

const keypairAccount = Account(provider, account.roochAddress, authorizer)

const result = keypairAccount.runFunction(
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::counter::increase',
  [],
  [],
  {
    maxGasAmount: 100000000,
  },
)
```

## Reading APIs

### Move view

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)

const result = provider.executeViewFunction(
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::counter::view',
)
```

##

### Get Transactions By Hash

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)

const allTransaction = provider.getTransactionsByHash([
  '0x70c42b134148cbe598b347c66574fc19f5a0fb6ee33df37255a96d8a8310c7a5',
])
```

### listTransactions

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)

const allTransaction = provider.getTransactionsByHash([
  '0x70c42b134148cbe598b347c66574fc19f5a0fb6ee33df37255a96d8a8310c7a5',
])
```

### Get State

Refer to [this storage guide](https://rooch.network/zh-CN/docs/dive-into-rooch/storage-abstraction) for more information.

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)

const state = provider.getStates('object/0x1')
```

### Get List States

Refer to [this storage guide](https://rooch.network/zh-CN/docs/dive-into-rooch/storage-abstraction) for more information.

```typescript
import { JsonRpcProvider, DevChain } from '@rooch/sdk'
const provider = new JsonRpcProvider(DevChain)

const states = provider.listStates('object/0x1', null, 10)
```

## Project Structure

The Rooch TypeScript SDK provides APIs and interfaces you can use to interact with the Rooch network for reading the blockchain state and for sending your transaction to the Rooch network.

The Rooch TypeScript SDK has three logical layers:

Plugins layer Implementation of different use cases such as Token etc.
Core layer – Exposes the functionalities needed by most applications.
Transport Layer Responsible on communication with the blockchain server.

See below a high-level architecture diagram of the Rooch TypeScript SDK.

## File Structure

```
├── examples                      // all the cases examples go into here
├── src                           // TODO:
└── test                          // e2e the test are in here
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
