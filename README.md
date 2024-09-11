<div width="400" align="center">
  <br />
  <br />
  <a href="https://rooch.network"><img alt="Rooch" src="https://rooch.network/logo/rooch_black_combine.svg" width=384></a>
  <br />
  <h3><a href="https://rooch.network">Rooch</a> is a VApp Container with <a href="https://github.com/move-language/move">Move</a> language for Bitcoin ecosystem.</h3>
  <br />
</div>

[![Check-Build-Test](https://github.com/rooch-network/rooch/actions/workflows/check_build_test.yml/badge.svg)](https://github.com/rooch-network/rooch/actions/workflows/check_build_test.yml)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
<!-- [![LoC](https://tokei.rs/b1/github/rooch-network/rooch?category=lines)](https://github.com/rooch-network/rooch) -->

**[VApp](https://rooch.network/learn/core-concepts/vapp)** stands for Verifiable Application. A Verifiable Application aims to ensure the verifiability of both computations and states within the application, thereby guaranteeing the transparency and reliability of its operations.


## Developer Network Information

* Name: dev
* ChainID: 3
* Bitcoin Network: regtest
* RPC: https://dev-seed.rooch.network/

Please refer to [Connect to Developer Test Network](https://rooch.network/build/getting-started/connect-to-rooch/connect-devnet) for more information.

## Developer Network Information

* Name: test
* ChainID: 2
* Bitcoin Network: test
* RPC: https://test-seed.rooch.network/

Please refer to [Connect to Test Network](https://rooch.network/build/getting-started/connect-to-rooch/connect-testnet) for more information.

## MoveStd & MoveosStd & RoochFramework documentation

* std: 0x1 [MoveStdlib](./frameworks/move-stdlib/doc)
* moveos_std: 0x2 [MoveosStdlib](./frameworks/moveos-stdlib/doc)
* rooch_framework: 0x3 [RoochFramework](./frameworks/rooch-framework/doc/)
* bitcoin_move: 0x4 [BitcoinMove](./frameworks/bitcoin-move/doc/)

Please refer to [Rooch's built-in library](https://rooch.network/learn/core-concepts/move-contracts/built-in-library) for more information.

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
   
5. Starting a local server
    ```bash
    rooch server start -n local
    ```
   * *`RUST_LOG=debug rooch server start` for debugging information*
   * You can directly use the devnet or testnet to skip this step.
   * If you want to program with Bitcoin UTXO or Inscription, please setup the local Bitcoin development env and start the server with:
    ```bash
    rooch server start -n local  --btc-rpc-url http://127.0.0.1:18443 --btc-rpc-username roochuser --btc-rpc-password roochpass --btc-sync-block-interval 1 
    ```
    * Please refer to  [Bitcoin local development environment](./scripts/bitcoin/README.md) for setup the local Bitcoin development env.


6. Publishing the Move project
    ```bash
    rooch move publish -p my_move_project
    ```
   
*Experience Rooch through [examples](examples).*

## Deep Dive into Rooch

<details>
<summary>Storage Abstraction</summary>

- [Docs](https://rooch.network/learn/core-concepts/objects/storage-abstraction)

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

- [Docs](https://rooch.network/learn/core-concepts/transaction/transaction-flow)
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
