#Benchmark

1. run all benchmark

```shell
cargo bench
```

2. run a special benchmark

```shell
cargo bench --bench bench_transaction
cargo bench --bench bench_transaction  -- --verbose
```

3. run a special benchmark with pprof (on linux)
```shell
cargo bench --bench bench_transaction -- --profile-time=10
```