Data Availability
===

**TODO**:

Merge to specification.

## Overview

In Rooch, DA (Data Availability) is the `input` in async states verification process:

verification:

1. `input` + `state transition function` = `actual output`
2. compare `actual output` with `expected output`

### Target

1. **Equivalent to Bitcoin Consensus**: Pack DA into Bitcoin block.
2. **Self-Verifying**: Anyone could verify DA by checksum and its signature.
3. **Open**: DA could be anywhere, anyone could access it without permission.

## Key Concepts

### DA Stream

[DA Stream](./docs/stream.md) is a continuous flow of data from sequencer to verifier. It is a sequence of DA Batch.

## Data Flow

![da-data-flow](../../docs/website/public/docs/da-data-flow-overview.svg)

In Verifier's perspective (verifier verifies tx), the data flow is as follows:

1. user submits tx to sequencer
2. sequencer buffers transactions to a batch for lower average latency & gas cost
3. sequencer puts batch to DA server
4. verifier get batch from DA server by:
    1. pull DA stream from DA server (after booking)
    2. get batch from DA server by batch hash/block number
    3. get segments from DA backend (after being submitted to DA backend)

## Roles

### Sequencer

Tx batch maker. Each sequencer maintains its own DA server.

### DA Server

Has responsibilities:

1. public DA Backend info.
2. provides Put/Get interface for DA.
3. Response to DA challenges.

Each DA server could connect to multiple DA backends.

## DA Backend

The purpose of DA backend is to mitigate the single point of risk associated with DA server. DA server,
not the backend, remains the principal party responsible for data publication. Therefore, the DA server may elect to
submit data to DA backend asynchronously.

## DA Server APIs

### Put

Put includes these actions:

1. Sequencer puts data to DA server
2. Sequencer packs data meta to Proposer
3. DA server put data to DA backends

### Get

There are various ways to get batch data. DA Batch could be verified by meta on Bitcoin.

#### Bypass DA server accessing DA Backend directly

Verifier could access DA backend directly to get data. However, it's not recommended because of the following reasons:

1. DA backend might lag behind the most recent data, given the likelihood of its data being uploaded asynchronously.
2. DA backend might be slow to respond to requests, DA server is the professional storage node.
3. DA server, accountable for data accessibility, risks forfeiture of its deposit via arbitration if it fails to meet
   the conditions of data availability.

This methodology may be employed to access data in the event that all DA servers are unable to respond appropriately.

#### Booking DA Stream by DA server（TODO）

Verifier subscribe to a data stream from the DA server.

#### Get DA Batch by DA server (TODO)

DA server maintains a batch index, which is updated in real time as new batches are added. Anyone could get batch.
