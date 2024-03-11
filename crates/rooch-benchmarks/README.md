# Benchmark

## Run

run all benchmark:

```shell
cargo bench
```

run specific benchmark:

```shell
cargo bench --bench bench_tx
cargo bench --bench bench_tx  -- --verbose
cargo bench --bench bench_tx_query
cargo bench --bench bench_tx_write
```

### Run with Args

#### bench_tx_write:

pass by env var:

1. PPROF_OUT: flamegraph(default), proto
2. TX_SIZE: 0(default)
3. TX_TYPE: empty(default), transfer, blog
4. DATA_DIR: `<rand in tmp dir>` (default)

for PPROF_OUT output location:

1. flamegraph: `rooch/target/criterion/execute_tx/profile/flamegraph.svg`
2. proto: `rooch/target/criterion/execute_tx/profile/profile.pb`

for proto, run these to get svg:

```shell
pprof -svg profile.pb
```

## Profiling

```shell
cargo bench --bench bench_tx_write -- --profile-time=3
```
