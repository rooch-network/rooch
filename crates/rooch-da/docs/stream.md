DA Stream
====

DA Stream is a continuous flow of data from sequencer to verifier. It is a sequence of DA Batch.

## Batch

A batch is a collection of transactions. It is the unit of data flow in DA Stream.

Each batch maps to a L2 block.

## Chunk

A chunk is a collection of DA Batch for better compression ratio.

Components:

- Chunk Header: Version, Chunk Number, Chunk Checksum
- Chunk Body: One or more DA batch after compression

Version of chunk determines:

1. chunk format: serialization/deserialization, compression algorithm of chunk body
2. segment format

### Segment

Segment consists of chunk bytes split by a certain size.

Segment is the unit submitted to DA backend, designed to comply with the block size restrictions of the DA backend.

Version of segment inherits from chunk version.