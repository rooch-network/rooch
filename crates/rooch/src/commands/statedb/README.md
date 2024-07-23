## Rooch Statedb tool

A tool to export and import rooch statedb.

### Usage

#### Prerequisite for genesis-* command

1. bitcoind synced with `-txindex=1` and `-server=1` option:

```shell
bitcoind -datadir=<datadir> -txindex=1 -server=1
```

2. set block height to genesis block height:

expect height: `<h>`

get `<h+1>` block hash:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> getblockhash <h+1>
<h+1 block hash>
```

invalid `<h+1>` block:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> invalidateblock <h+1 block hash>
```

check block height is expected:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> getblockcount
<h>
```

3. prepare utxo source file:

> - stop bitcoind first
> - clone chainstate:

```shell
rsync --delete -av <datadir/chainstate/> <chainstate_clone_path>
```

> - dump utxo source file(each line is a utxo record, format
    is `count,txid,vout,height,coinbase,amount,script,type,address`) by
    [bitcoin-utxo-dump](https://github.com/in3rsha/bitcoin-utxo-dump):

```shell
bitcoin-utxo-dump -f count,txid,vout,height,coinbase,amount,script,type,address -db <chainstate_clone_path> -o <output>
```

> - check max height of utxo dump file is <h> by python script:

```python
import pandas as pd
import argparse
import sys

def find_max_column_value_in_chunks(filename, column_name='height', chunksize=10000):
    """
    Read a CSV file in chunks and find the maximum value in the specified column

    :param filename: Path to the CSV file
    :param column_name: Name of the column to find the maximum value
    :param chunksize: Number of rows to read at a time
    :return: Maximum value in the specified column
    """
    try:
        max_value = None
        for chunk in pd.read_csv(filename, chunksize=chunksize):
            if column_name not in chunk.columns:
                raise ValueError(f"Column '{column_name}' does not exist in the file '{filename}'")
            current_max = chunk[column_name].max()
            if max_value is None or current_max > max_value:
                max_value = current_max
        return max_value

    except FileNotFoundError:
        print(f"The file '{filename}' does not exist")
        sys.exit(1)
    except pd.errors.EmptyDataError:
        print(f"The file '{filename}' is empty")
        sys.exit(1)
    except Exception as e:
        print(f"An error occurred: {e}")
        sys.exit(1)

if __name__ == "__main__":
    # Set up argument parser
    parser = argparse.ArgumentParser(description='Find the maximum value in a specified column of a CSV file')
    parser.add_argument('filename', type=str, help='Path to the CSV file')
    parser.add_argument('--column_name', type=str, default='height', help='Name of the column to find the maximum value (default is height)')
    parser.add_argument('--chunksize', type=int, default=10000, help='Number of rows to read at a time (default is 10000)')

    # Parse arguments
    args = parser.parse_args()

    # Find the maximum value in the specified column
    max_value = find_max_column_value_in_chunks(args.filename, args.column_name, args.chunksize)

    # Print the result
    print(f"The maximum value in column '{args.column_name}' is: {max_value}")
```

4. prepare ord source file(if needed):

> - start bitcoind again
> - dump ord source file by
    [ord](https://github.com/popcnt1/ord):

5. prepare genesis env:

```shell
rooch genesis init -n main -d <rooch_datadir>
```

#### Commands

**genesis-utxo**:

```shell
rooch statedb genesis-utxo --input <utxo_src_path> -d <rooch_datadir> -n main --batch-size <utxo_batch_size>
```

**genesis-ord**:

```shell
rooch statedb genesis-ord --utxo-source <utxo_src_path> --ord-source <ord_src_path> -d <rooch_datadir> -n main --utxo-ord-map <db_dir> --utxo-batch-size <utxo_batch_size> --ord-batch-size <ord_batch_size>
```

***tips***:

> - `--utxo-ord-map` is redb database file path. We could reuse it in `genesis-utxo` command.
> - `--batch-size`/`--utxo-batch-size` is optional, default is 2M. Set it smaller if memory is limited.
> - `--ord-batch-size` is optional, default is 1M. Set it smaller if memory is limited.

##### Export & Import

**TODO**:

1. export mode: full, indexer, genesis, snapshot are unfinished
2. recursion object export(export mode: object): export object and its children is supported; children's children cannot
   be exported

**rooch statedb export**:

```shell
rooch statedb export --output {your file} -d {your rooch data dir} -n main -m {export mode}
```

**rooch statedb import**:

```shell
rooch statedb import --input {your file} -d {your rooch data dir} -n main
```
