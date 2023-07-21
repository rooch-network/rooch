# Rooch TypeScript SDK

This is the Rooch TypeScript SDK built on the Rooch [JSON RPC API](https://github.com/rooch-network/rooch/blob/main/crates/rooch-open-rpc-spec/schemas/openrpc.json). It provides utility classes and functions for applications to sign transactions and interact with the Rooch network.

WARNING: Note that we are still iterating on the RPC and SDK API before TestNet, therefore please expect frequent breaking changes in the short-term. We expect the API to stabilize after the upcoming TestNet launch.

## Project Structure

The Rooch TypeScript SDK provides APIs and interfaces you can use to interact with the Rooch network for reading the blockchain state and for sending your transaction to the Rooch network.

The Rooch TypeScript SDK has three logical layers:

Plugins layer Implementation of different use cases such as Token etc.
Core layer – Exposes the functionalities needed by most applications.
Transport Layer Responsible on communication with the blockchain server.

See below a high-level architecture diagram of the Rooch TypeScript SDK.

|              | Client              |         |     |
| ------------ | ------------------- | ------- | --- |
| BCS          | Transaction Builder | Account |     |
| RPC Provider | Metamask Provider   |         |     |

## File Structure

```
├── examples                      // all the cases examples go into here
├── src                           // TODO:
│   └── test                      // all the test are in here
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

To run unit tests

```
pnpm test
```

To run E2E tests against local network

TODO:

## Working with local network

TODO:
