## Rooch DB tool

Toolkits for RoochDB.

### Usage

#### Revert

Revert the transaction by tx_order.

```shell
rooch db revert-tx --tx-order {tx_order} -d {data_dir} -n {network}
```

tx_order: The order of the transaction to be reverted. Must be last tx_order. If not, it will panic and print the last
tx_order.

#### Rollback

Rollback to tx_order.

```shell
rooch db rollback --tx-order {tx_order} -d {data_dir} -n {network}
```

#### Drop

Drop the database column family.

drop column family is a dangerous operation, make sure you know what you are doing

```shell
rooch db drop --cf-name {column_family} -d {data_dir} -n {network} --force true
```

re-create the column family after dropping for cleaning up the column family.

```shell
rooch db drop --cf-name {column_family} -d {data_dir} -n {network} --force true --re-create true
```