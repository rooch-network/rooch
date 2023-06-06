# KV store example
 
The purpose of this example is mainly to show use table as a KV store

## Getting started

> `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01` is a example address. When executing, please replace it with your own local address.

### Publish and init

```bash
rooch move publish -p examples/kv_store --sender-account default --named-addresses rooch_examples=default
```

### Add value

```bash
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::kv_store::add_value --args "b\"key1\"" "b\"value1\"" --sender-account default
```

### Get value

```bash
rooch move view --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::kv_store::get_value --args "b\"key1\""
```
```json
[
  {
    "value": {
      "type_tag": "0x1::string::String",
      "value": "0x0676616c756531"
    },
    "move_value": "value1"
  }
]
```

### Get KVStore resource

```bash
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::kv_store::KVStore
```
```json
[
  {
    "state": {
      "value": "0x7bffff7301cbcafe87a12a5a4e0f3798e9062743df6330eaa0ba2e92dd685a1e",
      "value_type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::kv_store::KVStore"
    },
    "move_value": {
      "abilities": 12,
      "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::kv_store::KVStore",
      "value": {
        "table": {
          "abilities": 4,
          "type": "0x1::table::Table<0x1::string::String, 0x1::string::String>",
          "value": {
            "handle": "0x7bffff7301cbcafe87a12a5a4e0f3798e9062743df6330eaa0ba2e92dd685a1e"
          }
        }
      }
    }
  }
]
```

### Get Table Items by key

> Table AccessPath: `/table/$table_handle`
> The table_handle from previous output. 

```bash
rooch state --access-path /table/0x7bffff7301cbcafe87a12a5a4e0f3798e9062743df6330eaa0ba2e92dd685a1e/key1
```
```json
[
  {
    "state": {
      "value": "0x0676616c756531",
      "value_type": "0x1::string::String"
    },
    "move_value": "value1"
  }
]
```