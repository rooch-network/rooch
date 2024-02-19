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

## On OSX

1. install xcode and command line tools

2. install cargo instruments
```shell
brew install cargo-instruments
```

3. install cargo flamegraph
```shell
cargo install flamegraph
```

4. install gnuplot
```shell
brew install gnuplot
```

5. run with profile
```shell
cargo instruments -t time --bench bench_transaction -- --bench
```