# Rooch DA tool

Toolkits for RoochDA.

## Usage

### namespace

Derive DA namespace from genesis file:

```shell
rooch da namespace --genesis-file-path {genesis-file}
```

### unpack

#### download segments

the easiest way to download segments is using `getda` tool from [here](https://github.com/popcnt1/roh) for downloading
segments from cloud storage.

```shell
getda --output={segment-dir} --url={da-cloud-storage-path} --last_chunk={max-chunk-id-expected} --max_goroutines={max-goroutines}
```

#### unpack segments

Unpack tx list from segments to human-readable format:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir}
```

If you want to verify tx list order, you can use `--verify-order` flag:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir} --verify-order
```

### exec

Execute tx list with state root verification(compare with Rooch Network Mainnet/Testnet).
It's a tool built for verification at development stage, not a full feature tool for sync states in production.

Features includes:

1. Execute tx list from segment dir and saving changes locally
2. Compare state root with Rooch Network Mainnet/Testnet by tx_order:state_root list file
3. We could collect performance data in execution process by tuning tools like `perf` if needed

#### Prepare tx_order:state_root list file

using [order_state.sh](https://github.com/popcnt1/roh/blob/main/scripts/order_state.sh) to generate tx_order:state_root
list file:

```shell
rooch env switch -n {network}
./order_state.sh {start_order} {end_order} {interval}
```

#### Prepare genesis

if you just import data by `rooch statedb genesis`, and will execute transaction from tx_order 1 (genesis tx is tx_order
0).

For ensuring everything works as expected, you should:

clean dirty genesis states:

```shell
rooch statedb re-genesis -d {data_dir} -n {network} --mode remove
```

genesis init(add builtin genesis back into db):

```shell
rooch genesis init -d {data_dir} -n {network}
```

we assume it's builtin genesis, because the target we want to verify is Rooch Network Mainnet/Testnet, all the two
Network are using builtin genesis.