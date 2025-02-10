DA Stream
====

DA Stream is a continuous flow of data from sequencer to verifier. It is a sequence of DA Batch.

All efforts on DA are to maintain a single trustable DA data stream with high-performance and low cost.

DA server is obligated to pay fees to DA backend and is subject to its interface restrictions.
Particularly in the forthcoming decentralized DA server cluster, faced with a variety of different DA backend
implementations,
we require the DA server to maintain flexibility and low cost in its implementation while providing a unified interface.
Rooch Network achieves our goals by treating the transaction sequence as a stream and flexibly dividing it into
segments:

1. Each network(e.g. main/test) has its own stream from genesis.
2. Several batches form a chunk for a better compression ratio.
3. Every chunk, once compressed, will be partitioned into numerous segments to comply with the block size restrictions
   of the DA backend. Simultaneously, this approach aids in augmenting parallelism.

## Batch

A batch is a collection of transactions. It is the unit of data flow in DA Stream.

Each batch maps to a range of tx.

## Chunk

A chunk is a collection of DA Batch for better compression ratio.

Components:

- Chunk Header: Version, Chunk Number, Chunk Checksum
- Chunk Body: One or more DA batch after compression

Version of chunk determines:

1. chunk format: serialization/deserialization, compression algorithm of chunk body
2. segment format

### Segment

A Segment consists of chunk bytes split by a certain size.

Segment is the unit submitted to DA backend, designed to comply with the block size restrictions of the DA backend.

Version of a segment inherited from its chunk version.