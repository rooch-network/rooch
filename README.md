<div align="center">
  <br />
  <br />
  <a href="https://rooch.network"><img alt="Rooch" src="https://rooch.network/logo/rooch_black_combine.svg" width=384></a>
  <br />
  <h3><a href="https://rooch.network">Rooch</a> is a Modular Fully on-chain Application Container, with <a href="https://github.com/move-language/move)">Move</a> language.</h3>
  <br />
</div>

[![Check-Build-Test](https://github.com/rooch-network/rooch/actions/workflows/check_build_test.yml/badge.svg)](https://github.com/rooch-network/rooch/actions/workflows/check_build_test.yml)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![LoC](https://tokei.rs/b1/github/rooch-network/rooch?category=lines)](https://github.com/rooch-network/rooch)

## Usage

* **RoochNetwork(Multi-Chain Modular Layer2)**: Rooch(Execution) + Multi-Chain(Settlement) + Ethereum(Arbitration) + DA
* **Sovereign Rollup**: Rooch + DA

## Getting Started

1. Building from source:
    ```bash
    cargo build && cp target/debug/rooch ~/.cargo/bin/
    ```

2. initialize Rooch config:
    ```bash
    rooch init
    ```
 
3. Creating a new Move project:
    ```bash
    rooch move new my_move_project
    ```
4. Building the Move project:
    ```bash
    cd my_move_project && rooch move build
    ```
   
5. Starting the server
    ```bash
    rooch server start
    ```
   * *`RUST_LOG=debug rooch server start` for debugging information*

6. Publishing the Move project
    ```bash
    rooch move publish -p my_move_project
    ```
   
*Experience Rooch through [examples](examples).*

## Deep Dive into Rooch

<details>
<summary>Storage Abstraction</summary>

- [Docs](./docs/website/pages/docs/tech-highlights/storage_abstraction.en-US.mdx)

- Overview:

![Storage Abstraction](./docs/website/public/docs/rooch-design-storage-abstraction.svg)
</details>

<details>
<summary>State DB</summary>

- Overview:

![State DB](./docs/website/public/docs/rooch-design-statedb.svg)

</details>

<details>
<summary>Transaction Flow</summary>

- [Docs](./docs/website/pages/docs/tech-highlights/transaction_flow.en-US.mdx)
- Overview:
![Rooch Transaction Flow](./docs/website/public/docs/rooch-design-transaction-flow-functional-perspective.svg)

</details>

## Components

* [MoveOS](./moveos): MoveOS is a standalone Move runtime environment based on [MoveVM](https://github.com/move-language/move). It provides Move execution environment for Rooch.

## Contributing

Rooch is an open source project, you can help with ideas, code, or documentation, we appreciate any efforts that help us to make the project better! 

To get started with contributing:

[The First Good Pull Request](./CONTRIBUTING.md)

## Community

* [Discord](https://discord.gg/rooch)
* [Twitter](https://twitter.com/RoochNetwork)
* [GitHub](https://github.com/rooch-network)

## License

Current Rooch code is released under [Apache 2.0](./LICENSE).

When contributing to a Rooch feature, you can find the relevant license in the comments at the top of each file.
