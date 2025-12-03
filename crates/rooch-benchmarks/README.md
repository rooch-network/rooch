# Benchmark

## TODO

1. refine bench_tx_query, is too heavy cannot distinguish the performance of different tx_types/components.
2. add more bench
3. pass args from command line not env var

## Usage

run all benchmark:

```shell
cargo bench
```

run specific benchmark:

1. check the benchmark name in `Cargo.toml`:

```toml
[[bench]]
harness = false
name = "benchmark_name"
```

2. run it:

```shell
cargo bench --bench benchmark_name
```

### Options

Some benchmark can be configured by env var and config file.

For all benchmarks of transactions we have `ROOCH_TEST_DATA_DIR` env var to specify the data dir. Default is temp dir.

For `bench_tx_exec`  we have `ROOCH_BENCH_TX_CONFIG_PATH` env var to specify the config file. Default
is `rooch-benchmarks/config/bench_tx.toml`.

```rust
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Parser, Eq)]
pub struct BenchTxConfig {
    pub tx_type: Option<TxType>,    // empty(default)/transfer/btc-block
    pub btc_block_dir: Option<String>, // btc block dir, default: target/btc_blocks, file name: <height>.hex
    pub btc_block_start_height: Option<u64>, // btc block start height, default: 820000
    pub btc_rpc_url: Option<String>,
    pub btc_rpc_username: Option<String>,
    pub btc_rpc_password: Option<String>,
    pub pprof_output: Option<PProfOutput>, // flamegraph(default)/proto
}
```

The env var has higher priority than the config file.

* `ROOCH_BENCH_TX_TYPE`: override `tx_type` in config file.
* `ROOCH_BENCH_BTC_BLOCK_DIR`: override `btc_block_dir` in config file.
* `ROOCH_BENCH_BTC_BLOCK_START_HEIGHT`: override `btc_block_start_height` in config file.
* `ROOCH_BENCH_BTC_RPC_URL`: override `btc_rpc_url` in config file.
* `ROOCH_BENCH_BTC_RPC_USERNAME`: override `btc_rpc_username` in config file.
* `ROOCH_BENCH_BTC_RPC_PASSWORD`: override `btc_rpc_password` in config file.
* `ROOCH_BENCH_PPROF_OUTPUT`: override `pprof_output` in config file.

## Profiling

When your run bench with `-- --profile-time=<seconds>` option, it will generate a flamegraph file
in `target/criterion/<bench_group_name>/<bench_id>/profile` dir.

e.g., profiling `bench_tx_exec` for 3.1 seconds:

```shell
cargo bench --bench bench_tx_exec -- --profile-time=3
```

for PPROF_OUT output location:

1. flamegraph: `rooch/target/criterion/bench_tx_exec/<tx_type>/profile/flamegraph.svg`
2. proto: `rooch/target/criterion/bench_tx_exec/<tx_type>/profile/profile.pb`

`<tx_type>`:

1. `l2_tx_<transfer/empty/transfer_large_object>`
2. `btc_block`
3. `btc_tx`

for proto, run these to get svg:

```shell
pprof -svg profile.pb
```

## FAQ

### Why not run in CI pipeline?

Coming soon...

### How to prepare the Bitcoin blocks

Run the benchmark with Bitcoin RPC config, it will download the blocks from Bitcoin network and save them in
`target/btc_blocks` dir.

```shell
ROOCH_BENCH_TX_TYPE=btc_tx ROOCH_BENCH_BTC_RPC_URL=http://localhost:8332 ROOCH_BENCH_BTC_RPC_USERNAME=YourBTCUser ROOCH_BENCH_BTC_RPC_PASSWORD=YourBTCPass cargo bench -p rooch-benchmarks --bench bench_tx_exec
```

## GC Mark Phase Benchmark

Benchmark suite for testing GC (Garbage Collector) Mark Phase performance, focusing on:
- **AtomicBloomFilter vs Mutex BloomFilter**: Lock-free atomic operations vs mutex-based implementation
- **Parallel vs Single-threaded**: Work-stealing parallel algorithm efficiency
- **Worker count impact**: Performance with different thread counts
- **Batch size impact**: Batch processing size effect on I/O performance
- **Multi-root scenarios**: Parallel traversal of multiple root nodes

### Run GC Benchmarks

```bash
# Run all GC benchmarks
cargo bench --bench bench_gc

# Run specific benchmark groups
cargo bench --bench bench_gc -- mark_phase_scaling        # Data scaling (10K, 50K, 100K nodes)
cargo bench --bench bench_gc -- mark_phase_workers        # Worker count (1, 2, 4, 8)
cargo bench --bench bench_gc -- mark_phase_batch_size     # Batch size (100-10000)
cargo bench --bench bench_gc -- atomic_bloom_comparison   # AtomicBloom vs Mutex Bloom
cargo bench --bench bench_gc -- mark_phase_multi_root     # Multi-root parallel
```

### GC Benchmark Groups

1. **mark_phase_scaling** - Tests performance at different data scales (10K, 50K, 100K nodes)
2. **mark_phase_workers** - Tests worker count impact (1, 2, 4, 8 workers) with 100K nodes
3. **mark_phase_batch_size** - Tests batch size optimization (100-10000) with 100K nodes and 4 workers
4. **atomic_bloom_comparison** - Compares AtomicBloomFilterMarker vs BloomFilterMarker with 50K nodes
5. **mark_phase_multi_root** - Tests 4 independent trees (10K nodes each) in single-thread vs 4-worker parallel

### View Results

Benchmark results are saved in `target/criterion/<benchmark_name>/report/index.html`

```bash
open target/criterion/mark_phase_scaling/report/index.html
```

### Key Optimizations

- **AtomicBloomFilterMarker**: Uses `AtomicU8` and atomic operations instead of Mutex, reducing lock contention
- **Work-Stealing**: Implements work-stealing using `crossbeam-deque` for better load balancing
- **Batch I/O**: Uses `multi_get()` to batch-read nodes, reducing database access count
- **Overflow Strategy**: Overflows to global queue when local queue is full, enabling work stealing