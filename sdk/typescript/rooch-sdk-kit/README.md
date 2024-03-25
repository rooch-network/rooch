# Rooch TypeScript SDK Kit

The rooch-sdk-kit is a set of React components, hooks, and utilities that make it easy to build a
dApp for the Rooch ecosystem. It provides hooks and components for querying data from the rooch
blockchain, and connecting to wallets.

### Core Features

- **Query Hooks:** rooch-sdk-kit provides a set of hooks for making rpc calls to the rooch blockchain,
  making it easy to load any information needed for your dApp.
- **Automatic Wallet State Management:** rooch-sdk-kit removes the complexity of state management related
  to wallet connections. You can focus on building your dApp.
- **Supports wallets:** bitcoin: unisat,okx. eth: matamask. All support wallets are supported rooch chain.
- **Flexible:** rooch-sdk-kit ships lower level hooks that you can use to build your own custom components.

## Installation

```shell
npm i --save @roochnetwork/rooch-sdk-kit @roochnetwork/rooch-sdk @tanstack/react-query
```

## Setting up providers

To be able to use the hooks and components in the rooch-sdk-Kit, you need to wrap your app with a couple
providers. The props available on the providers are covered in more detail in their respective docs
pages.

```tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { DevNetwork } from '@roochnetwork/rooch-sdk'
import { WalletProvider, RoochClientProvider, SupportChain } from '@roochnetwork/rooch-sdk-kit'

const queryClient = new QueryClient()

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RoochClientProvider defaultNetwork={DevNetwork}>
        <WalletProvider chain={SupportChain.BITCOIN} autoConnect>
          <YouApp />
        </WalletProvider>
      </RoochClientProvider>
    </QueryClientProvider>
  )
}
```

## Using hooks to make RPC calls

The rooch-sdk-kit provides a set of hooks for making RPC calls to the rooch blockchain. The hooks are thin
wrappers around `useQuery` from `@tanstack/react-query`. For more comprehensive documentation on how
these query hooks can be used, check out the
[react-query docs](https://tanstack.com/query/latest/docs/react/overview).

```tsx
import { useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit'

function MyComponent() {
  let { data, isPending, error } = useRoochClientQuery('getStates', '/object/0x1')

  if (isPending) {
    return <div>Loading...</div>
  }

  return <pre>{JSON.stringify(data, null, 2)}</pre>
}
```

## Building Locally

To get started you need to install [pnpm](https://pnpm.io/), then run the following command:

```bash

# Install all dependencies
$ pnpm install
# Run the build for the TypeScript SDK
$ pnpm rooch-sdk-kit build
# Run the build for the TypeScript SDK Kit
$ pnpm rooch-sdk build
```

> All `pnpm` commands are intended to be run in the root of the Rooch repo. You can also run them within the `sdk/typescript` directory, and remove change `pnpm sdk` to just `pnpm` when running commands.
