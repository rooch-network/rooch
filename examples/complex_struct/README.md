# Complex Struct example
 
The purpose of this example is mainly to test how types of MoveValue are displayed in the JSON-RPC output

## Getting started

> `0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c` is a example address. When executing, please replace it with your own local address.

### Publish and init

```bash
rooch move publish -p ./examples/complex_struct --sender-account 0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c --named-addresses rooch_examples=0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c
```

### Call the view function

```bash
rooch move view --function 0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::value
```
```json
{
  "vm_status": "Executed",
  "return_values": [
    {
      "value": {
        "type_tag": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::ComplexStruct",
        "value": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0f000000000000000000000000be032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c01f01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef86801f01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef86806e4bda0e5a5bd0100000000000000000000000000000000000000000000000000000000000000022a00000000000000012a00000000000000000106e4bda0e5a5bd00012a000000000000000020be032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c020100ffff0201000000ffffffff020100000000000000ffffffffffffffff020568656c6c6f06e4bda0e5a5bd012a00000000000000000101f01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef868"
      },
      "decoded_value": {
        "abilities": 12,
        "type": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::ComplexStruct",
        "value": {
          "field_address": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c",
          "field_decimal_value": 0.01,
          "field_object": {
            "abilities": 12,
            "type": "0x2::object::Object<0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct>",
            "value": {
              "id": "0xf01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef868"
            }
          },
          "field_object_id": "0xf01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef868",
          "field_option_str_none": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "field_option_str_some": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": {
                "abilities": 7,
                "type": "0x1::string::String",
                "field": [
                  "bytes"
                ],
                "value": [
                  [
                    "0xe4bda0e5a5bd"
                  ]
                ]
              }
            }
          },
          "field_option_struct_none": {
            "abilities": 7,
            "type": "0x1::option::Option<0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct>",
            "value": {
              "vec": []
            }
          },
          "field_option_struct_some": {
            "abilities": 7,
            "type": "0x1::option::Option<0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct>",
            "value": {
              "vec": {
                "abilities": 15,
                "type": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct",
                "field": [
                  "value"
                ],
                "value": [
                  [
                    "42"
                  ]
                ]
              }
            }
          },
          "field_option_u64_none": {
            "abilities": 7,
            "type": "0x1::option::Option<u64>",
            "value": {
              "vec": []
            }
          },
          "field_option_u64_some": {
            "abilities": 7,
            "type": "0x1::option::Option<u64>",
            "value": {
              "vec": [
                "42"
              ]
            }
          },
          "field_str": "你好",
          "field_struct": {
            "abilities": 15,
            "type": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct",
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
          "field_vec_object_ids": {
            "abilities": 7,
            "type": "0x2::object::ObjectID",
            "field": [
              "path"
            ],
            "value": [
              [
                [
                  "0xf01cda6ea59c27dc1dc3939dca70566abc54c469d894c8ec393f257ff53ef868"
                ]
              ]
            ]
          },
          "field_vec_str": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x68656c6c6f"
              ],
              [
                "0xe4bda0e5a5bd"
              ]
            ]
          },
          "field_vec_struct": {
            "abilities": 15,
            "type": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::SimpleStruct",
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
          "field_vec_u8": "0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c"
        }
      }
    }
  ]
}
```

### Show the resource

```bash
rooch state --access-path /resource/0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c/0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::ComplexStruct
```

### Show the object

```bash
rooch object -t 0xbe032a391b8df17f414c2d42cd0307c7c6e1ac5e047675f93935f6e5b3a3467c::complex_struct::ComplexStruct
```