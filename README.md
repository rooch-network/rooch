# MoveOS

MoveOS is a standalone Move runtime environment based on [MoveVM](https://github.com/move-language/move).

## Key Features

It provides the following features on top of MoveVM to make it easy for applications to integrate with the Move runtime environment. 

1. State storage and retrieval: it provides a default local database storage, which can be customized by applications.
2. Rust-To-Move extension point ABI: the application can implement the core logic through Move, and invoke in Rust, let application easy to maintain and upgrade. Such as transaction verification logic (Account Abstraction).
3. State proof: it provides state proof based on state tree (two-level smt).
4. Fraud proof: it provides interactive fraud proof based on [OMO](https://github.com/rooch-network/omo), which is necessary for modular applications. Zero-knowledge proofs based on [zkMove](https://github.com/young-rocks/zkmove) will be integrated in the future.

## Usage

1. as a rust library embedded in a blockchain or other application.
2. as a standalone process, called via REST API or IPC.

## Getting Started

1. Build from source
    $ cargo build && cp target/debug/mos ~/.cargo/bin/
2. Create a new Move project
    $ mos new my_mos_project
3. Build the Move project
    $ cd my_mos_project && mos build