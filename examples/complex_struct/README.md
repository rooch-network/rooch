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
        "value": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab017b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f017b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f06e4bda0e5a5bd0100000000000000000000000000000000000000000000000000000000000000022a00000000000000012a00000000000000000106e4bda0e5a5bd00012a00000000000000002045e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a000000000000000001017b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f"
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
              "id": "0x7b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f"
            }
          },
          "field_object_id": "0x7b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f",
          "field_option_str_none": null,
          "field_option_str_some": "你好",
          "field_option_struct_none": null,
          "field_option_struct_some": {
            "abilities": 15,
            "type": "0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::SimpleStruct",
            "value": {
              "value": "42"
            }
          },
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
            "0x7b73e5e9fc340b850da6c773e2025a2a5df7d8c9ee4e91302adf6e12edf5e62f"
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

### Show the object

```bash
rooch object -t 0x45e8af026e02c8efe11f1bc12ee05645175fb90922656956c0ccfe5c6f699cab::complex_struct::ComplexStruct
```