# Rooch TypeScript SDK

This is the Rooch TypeScript SDK built on the Rooch [JSON RPC API](https://github.com/rooch-network/rooch/blob/main/crates/rooch-open-rpc-spec/schemas/openrpc.json). It provides utility classes and functions for applications to sign transactions and interact with the Rooch network.

WARNING: Note that we are still iterating on the RPC and SDK API before TestNet, therefore please expect frequent breaking changes in the short-term. We expect the API to stabilize after the upcoming TestNet launch.

## Installation

```shell
npm i @roochnetwork/rooch-sdk
```

## Connecting to Rooch Network

The JsonRpcProvider class provides a connection to the JSON-RPC Server and should be used for all read-only operations. The default URLs to connect with the RPC server are:

- local: http://127.0.0.1:6767
- DevNet: https://dev-seed.rooch.network::443
- TestNet: https://test-seed.rooch.network

For local development, you can run cargo run server start to start a local network. Refer to this guide for more information.

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

You can also construct your own in custom connections, with the URL for your own network

```typescript
import { RoochClient } from '@roochnetwork/rooch-sdk'

const client = new RoochClient({
  url: 'http://127.0.0.1:6767',
})

// get balances
await client.getBalances({
  owner: '',
})
```

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

const result = provider.executeViewFunction(
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::counter::view',
)
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
