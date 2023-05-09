# Sparse Merkle Tree

A Rust library that implements a Sparse Merkle tree for a key-value store, provide state proof for the storage layer.

The origin source from [Starcoin](https://github.com/starcoinorg/starcoin/blob/f9ba8b637bade2eb38ae9e62a7e75f2c18ce7289/commons/forkable-jellyfish-merkle) and [Diem](https://github.com/diem/diem/tree/4eb8093bb190c1dca3706d9a7226a39f2089ef7a/storage/jellyfish-merkle). The tree's optimisations specified in the [Diem whitepaper](https://diem-developers-components.netlify.app/papers/the-diem-blockchain/2020-05-26.pdf).


This repository does or will do the following improvements:

1. released as a standalone crate for other projects to depend on.
2. refactor the interface to make it easier to use.
4. customize hash methods.
5. customize encode/decode methods.