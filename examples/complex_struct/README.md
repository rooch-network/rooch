# Complex Struct example
 
The purpose of this example is mainly to test how different types of MoveValue are displayed in the JSON-RPC output

## Getting started

> `0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91` is a example address. When executing, please replace it with your own local address.

### publish and init

```bash
rooch move publish -p ./examples/complex_struct --sender-account 0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91 --named-addresses rooch_examples=0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91
```

### Show the resource

```bash
rooch state --access-path /resource/0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91/0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct
```
```json
[
  {
    "state": {
      "value": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000040ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da9100c83391cba13a30ef3e961c5d940e424418ffcf4751b9310608113b58e21f160568656c6c6f06e4bda0e5a5bd2a000000000000002040ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a00000000000000",
      "value_type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct"
    },
    "move_value": {
      "abilities": 12,
      "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct",
      "value": {
        "field_address": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91",
        "field_ascii_str": "hello",
        "field_object_id": "0xc83391cba13a30ef3e961c5d940e424418ffcf4751b9310608113b58e21f16",
        "field_str": "你好",
        "field_struct": {
          "abilities": 7,
          "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::SimpleStruct",
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
        "field_vec_str": [
          "hello",
          "你好"
        ],
        "field_vec_struct": [
          {
            "abilities": 7,
            "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::SimpleStruct",
            "value": {
              "value": "42"
            }
          }
        ],
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
        "field_vec_u8": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91"
      }
    }
  }
]
```

### Show the object

> The `object_id` is the `field_object_id`'s value of previous output.

```bash
rooch state --access-path /object/0xc83391cba13a30ef3e961c5d940e424418ffcf4751b9310608113b58e21f16
```
```json
[
  {
    "state": {
      "value": "0x00c83391cba13a30ef3e961c5d940e424418ffcf4751b9310608113b58e21f1640ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f00000000000000000000000040ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da914a880af36118b1ea6f1d524e6ae2bbb7eb8c39c9814314fc444db6f19817208d0568656c6c6f06e4bda0e5a5bd2a000000000000002040ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a00000000000000",
      "value_type": "0x1::object::Object<0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct>"
    },
    "move_value": {
      "abilities": 0,
      "type": "0x1::object::Object<0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct>",
      "value": {
        "id": "0xc83391cba13a30ef3e961c5d940e424418ffcf4751b9310608113b58e21f16",
        "owner": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91",
        "value": {
          "abilities": 12,
          "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::ComplexStruct",
          "value": {
            "field_address": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91",
            "field_ascii_str": "hello",
            "field_object_id": "0x4a880af36118b1ea6f1d524e6ae2bbb7eb8c39c9814314fc444db6f19817208d",
            "field_str": "你好",
            "field_struct": {
              "abilities": 7,
              "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::SimpleStruct",
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
            "field_vec_str": [
              "hello",
              "你好"
            ],
            "field_vec_struct": [
              {
                "abilities": 7,
                "type": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91::complex_struct::SimpleStruct",
                "value": {
                  "value": "42"
                }
              }
            ],
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
            "field_vec_u8": "0x40ad744b6a3cd57d800e595b0aa9bf29ae0d2c1dadc78860a2c57df3cea5da91"
          }
        }
      }
    }
  }
]
```