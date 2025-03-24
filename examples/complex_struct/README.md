# Complex Struct example
 
The purpose of this example is mainly to test how types of MoveValue are displayed in the JSON-RPC output

## Getting started

> `0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab` is a example address. When executing, please replace it with your own local address.

### Publish and init

```bash
rooch move publish -p ./examples/complex_struct --sender-account 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab --named-addresses rooch_examples=0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab
```

### Call the view function

```bash
rooch move view --function 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::value
```
```json
{
  "vm_status": "Executed",
  "return_values": [
    {
      "value": {
        "type_tag": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
        "value": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec106e4bda0e5a5bd0100000000000000000000000000000000000000000000000000000000000000022a00000000000000012a00000000000000000106e4bda0e5a5bd002045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a000000000000000001013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1"
      },
      "decoded_value": {
        "abilities": 12,
        "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
        "value": {
          "field_address": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab",
          "field_decimal_value": 0.01,
          "field_object": {
            "abilities": 12,
            "type": "0x2::object::Object<0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct>",
            "value": {
              "id": "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1"
            }
          },
          "field_object_id": "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1",
          "field_option_str_none": null,
          "field_option_str_some": "你好",
          "field_option_u64_none": null,
          "field_option_u64_some": "42",
          "field_str": "你好",
          "field_struct": {
            "abilities": 15,
            "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
            "value": {
              "value": "42"
            }
          },
          "field_u128": "340282366920938463463374607431768211455",
          "field_u16": 65535,
          "field_u256": "91343852333181432387730302044767688728495783935",
          "field_u32": 4294967295,
          "field_u64": "18446744073709551615",
          "field_u8": 255,
          "field_vec_object_ids": [
            "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1"
          ],
          "field_vec_str": [
            "hello",
            "你好"
          ],
          "field_vec_struct": {
            "abilities": 15,
            "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
            "field": [
              "value"
            ],
            "value": [
              [
                "42"
              ]
            ]
          },
          "field_vec_struct_empty": [],
          "field_vec_u16": [
            1,
            65535
          ],
          "field_vec_u32": [
            1,
            4294967295
          ],
          "field_vec_u64": [
            "1",
            "18446744073709551615"
          ],
          "field_vec_u8": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab"
        }
      }
    }
  ]
}
```

### Show the resource

```bash
rooch state --access-path /resource/0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab/0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct
```
```json
[
  {
    "id": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab65be2406051e67a8cdc5d706504898b2f22d949ecad19cb14a1f34e9769d5b2e",
    "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
    "owner_bitcoin_address": null,
    "flag": 0,
    "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
    "size": "0",
    "created_at": "1742821068258",
    "updated_at": "1742821068258",
    "object_type": "0x2::object::DynamicField<0x1::string::String, 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct>",
    "value": "0x613078343565386166303236653032633865666531316631626331326565303536343531373566623930393232363536393536633063636665356336663639396361623a3a636f6d706c65785f7374727563743a3a436f6d706c6578537472756374ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec106e4bda0e5a5bd0100000000000000000000000000000000000000000000000000000000000000022a00000000000000012a00000000000000000106e4bda0e5a5bd002045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a000000000000000001013b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1",
    "decoded_value": {
      "abilities": 12,
      "type": "0x2::object::DynamicField<0x1::string::String, 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct>",
      "value": {
        "name": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
        "value": {
          "abilities": 12,
          "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
          "value": {
            "field_address": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab",
            "field_decimal_value": 0.01,
            "field_object": {
              "abilities": 12,
              "type": "0x2::object::Object<0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct>",
              "value": {
                "id": "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1"
              }
            },
            "field_object_id": "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1",
            "field_option_str_none": null,
            "field_option_str_some": "你好",
            "field_option_u64_none": null,
            "field_option_u64_some": "42",
            "field_str": "你好",
            "field_struct": {
              "abilities": 15,
              "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
              "value": {
                "value": "42"
              }
            },
            "field_u128": "340282366920938463463374607431768211455",
            "field_u16": 65535,
            "field_u256": "91343852333181432387730302044767688728495783935",
            "field_u32": 4294967295,
            "field_u64": "18446744073709551615",
            "field_u8": 255,
            "field_vec_object_ids": [
              "0x3b485eebd19ca1b547ef48a764e3eace04e72b4d8b124cac3b5270f75e248ec1"
            ],
            "field_vec_str": [
              "hello",
              "你好"
            ],
            "field_vec_struct": {
              "abilities": 15,
              "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
              "field": [
                "value"
              ],
              "value": [
                [
                  "42"
                ]
              ]
            },
            "field_vec_struct_empty": [],
            "field_vec_u16": [
              1,
              65535
            ],
            "field_vec_u32": [
              1,
              4294967295
            ],
            "field_vec_u64": [
              "1",
              "18446744073709551615"
            ],
            "field_vec_u8": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab"
          }
        }
      }
    },
    "display_fields": null
  }
]
```

### Show the object

> The `object_id` is new the `New objects` output of the publish output.

```
objectId: 0x29ccbbbb5199165006018f447460205248dae71c0591e51496826d2becb41c48
type: 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct
```

```bash
rooch object -i 0x29ccbbbb5199165006018f447460205248dae71c0591e51496826d2becb41c48
```
```json
{
  "data": [
    {
      "id": "0x29ccbbbb5199165006018f447460205248dae71c0591e51496826d2becb41c48",
      "owner": "rooch1gh527qnwqtywlcglr0qjaczkg5t4lwgfyfjkj4kqenl9cmmfnj4sw89vyq",
      "owner_bitcoin_address": "bcrt1p9wv3fg84rmjfrdst7wqq28t2wzn3gw86ns2j7xvvu849qpvaffrq8sq6pl",
      "flag": 0,
      "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
      "size": "0",
      "created_at": "1742821068258",
      "updated_at": "1742821068258",
      "object_type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
      "value": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab0128b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be0128b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be06e4bda0e5a5bd0100000000000000000000000000000000000000000000000000000000000000022a00000000000000012a00000000000000000106e4bda0e5a5bd002045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a0000000000000000010128b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be",
      "decoded_value": {
        "abilities": 12,
        "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct",
        "value": {
          "field_address": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab",
          "field_decimal_value": 0.01,
          "field_object": {
            "abilities": 12,
            "type": "0x2::object::Object<0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct>",
            "value": {
              "id": "0x28b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be"
            }
          },
          "field_object_id": "0x28b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be",
          "field_option_str_none": null,
          "field_option_str_some": "你好",
          "field_option_u64_none": null,
          "field_option_u64_some": "42",
          "field_str": "你好",
          "field_struct": {
            "abilities": 15,
            "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
            "value": {
              "value": "42"
            }
          },
          "field_u128": "340282366920938463463374607431768211455",
          "field_u16": 65535,
          "field_u256": "91343852333181432387730302044767688728495783935",
          "field_u32": 4294967295,
          "field_u64": "18446744073709551615",
          "field_u8": 255,
          "field_vec_object_ids": [
            "0x28b1094c8c192140eda1f618969cb564e63d7ea6b8f7d1a125ac0c17132099be"
          ],
          "field_vec_str": [
            "hello",
            "你好"
          ],
          "field_vec_struct": {
            "abilities": 15,
            "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
            "field": [
              "value"
            ],
            "value": [
              [
                "42"
              ]
            ]
          },
          "field_vec_struct_empty": [],
          "field_vec_u16": [
            1,
            65535
          ],
          "field_vec_u32": [
            1,
            4294967295
          ],
          "field_vec_u64": [
            "1",
            "18446744073709551615"
          ],
          "field_vec_u8": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab"
        }
      },
      "tx_order": "0",
      "state_index": "0",
      "display_fields": null
    }
  ],
  "next_cursor": {
    "tx_order": "0",
    "state_index": "0"
  },
  "has_next_page": false
}
```