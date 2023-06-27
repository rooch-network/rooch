# Rooch

[Rooch](https://rooch.network) is a modular DApp container with the [Move language](https://github.com/move-language/move).

## Usage

1. Rooch Ethereum Layer2: Rooch(Execution) + Layer1s(Settlement) + Ethereum(Arbitration) + DA
2. XChain Modular DApp: Rooch(Execution) + XChain(Settlement + Arbitration) + DA
3. Rooch Layer3 Modular DApp: Rooch(Execution) + Rooch Layer2(Settlement + Arbitration) + DA
4. Sovereign Rollup: Rooch + DA

## Getting Started

1. Build from source
    ```bash
    cargo build && cp target/debug/rooch ~/.cargo/bin/
    ```
2. Create a new Move project
    ```bash
    rooch move new my_move_project
    ```
3. Init Rooch config
    ```bash
    rooch init
    ```
4. Build the Move project
    ```bash
    cd my_move_project && rooch move build
    ```
5. Start the server
    ```bash
    rooch server start
    ```
> If you want a more detailed log for debugging, you can use `RUST_LOG=debug rooch server start`
6. Publish the Move project
    ```bash
    rooch move publish -p my_move_project
    ```

## Deep Dive into Rooch

### [Storage Abstraction](./docs/website/pages/docs/tech-highlights/storage_abstraction.en-US.mdx)

Storage Abstraction module overview:

![Storage Abstraction](./docs/website/public/docs/rooch-design-storage-abstraction.svg)

State DB:

![State DB](./docs/website/public/docs/rooch-design-statedb.svg)

### [Transaction Flow](./docs/website/pages/docs/tech-highlights/transaction_flow.en-US.mdx)

![Rooch Transaction Flow](./docs/website/public/docs/rooch-design-transaction-flow-functional-perspective.svg)

## Components

* [MoveOS](./moveos): MoveOS is a standalone Move runtime environment based on [MoveVM](https://github.com/move-language/move). It provides Move execution environment for rooch.

## How to Contribute

You can learn more about contributing to the Rooch project by reading our [Contribution Guide](./CONTRIBUTING.md) and by viewing our [Code of Conduct](./CODE_OF_CONDUCT.md).

Rooch Network Rooch is licensed under [Apache 2.0](./LICENSE).

## Join the Community

To connect with the Rooch Network community, please join our [Discord](https://discord.gg/rooch).
