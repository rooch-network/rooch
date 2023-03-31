# Rooch

[Rooch](https:://rooch.network) is the first modular Layer 2 solution with the Move language, using Ethereum as the security layer. Our target is to provide a Move-based execution module to all Web3 application scenarios as well as be a bridge between all Web3 developers and versatile crypto ecosystems.

## Usage
1. Rooch Ethereum Layer2: Rooch(Execution) + Layer1s(Settlement) + Ethereum(Arbitration) + DA
2. XChain Modular DApp: Rooch(Execution) + XChain(Settlement + Arbitration) + DA
3. Rooch Layer3 Modular DApp: Rooch(Execution) + Rooch Layer2(Settlement + Arbitration) + DA
4. Sovereign Rollup: Rooch + DA

## Components
* [MoveOS](./moveos): MoveOS is a standalone Move runtime environment based on [MoveVM](https://github.com/move-language/move). It provide Move execution environment for rooch.
* Sequencer(TODO)
* Proposer(TODO)
* Challenger(TODO)

## Getting Started

1. Build from source
    $ cargo build && cp target/debug/rooch ~/.cargo/bin/
2. Create a new Move project
    $ rooch move new my_move_project
3. Build the Move project
    $ cd my_move_project && rooch move build

## How to Contribute

You can learn more about contributing to the Rooch project by reading our [Contribution Guide](./CONTRIBUTING.md) and by viewing our [Code of Conduct](./CODE_OF_CONDUCT.md).

Rooch Network Rooch is licensed under [Apache 2.0](./LICENSE).

## Join the Community

To connect with the Rooch Network community, please join our [Discord](https://discord.gg/rooch).