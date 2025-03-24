// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    decimal_value_view::DecimalValueView,
    move_types::{AnnotatedMoveValueView, SpecificStructView},
    BytesView, StrView,
};
use move_binary_format::file_format::AbilitySet;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    u256,
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use moveos_types::{
    move_std::string::MoveString, moveos_std::decimal_value::DecimalValue, state::MoveStructType,
};
use serde_json::{json, Value};

#[test]
fn test_annotated_move_value_view_primitives() {
    // Test basic primitive types
    let cases = vec![
        (
            AnnotatedMoveValue::U8(42),
            json!(42),
            AnnotatedMoveValueView::U8(42),
        ),
        (
            AnnotatedMoveValue::U16(1234),
            json!(1234),
            AnnotatedMoveValueView::U16(1234),
        ),
        (
            AnnotatedMoveValue::U16(u16::MAX),
            json!(u16::MAX),
            AnnotatedMoveValueView::U16(u16::MAX),
        ),
        (
            AnnotatedMoveValue::U32(12345678),
            json!(12345678),
            AnnotatedMoveValueView::U32(12345678),
        ),
        (
            AnnotatedMoveValue::U64(1234567890),
            json!("1234567890"),
            AnnotatedMoveValueView::U64(StrView(1234567890)),
        ),
        (
            AnnotatedMoveValue::U128(12345678901234567890),
            json!("12345678901234567890"),
            AnnotatedMoveValueView::U128(StrView(12345678901234567890)),
        ),
        (
            AnnotatedMoveValue::U256(u256::U256::from(123456789012345678901234567890u128)),
            json!("123456789012345678901234567890"),
            AnnotatedMoveValueView::U256(StrView(u256::U256::from(
                123456789012345678901234567890u128,
            ))),
        ),
        (
            AnnotatedMoveValue::Bool(true),
            json!(true),
            AnnotatedMoveValueView::Bool(true),
        ),
        (
            AnnotatedMoveValue::Address(AccountAddress::from_hex_literal("0x1").unwrap()),
            json!("0x0000000000000000000000000000000000000000000000000000000000000001"),
            AnnotatedMoveValueView::Address(StrView(
                AccountAddress::from_hex_literal("0x1").unwrap(),
            )),
        ),
        (
            AnnotatedMoveValue::Bytes(vec![1, 2, 3, 4]),
            json!("0x01020304"),
            AnnotatedMoveValueView::Bytes(BytesView::from(vec![1, 2, 3, 4])),
        ),
    ];

    for (input, expected_json, expected_view) in cases {
        let view: AnnotatedMoveValueView = input.into();
        assert_eq!(view, expected_view);
        let serialized = serde_json::to_value(&view).unwrap();
        assert_eq!(serialized, expected_json);
    }
}

#[test]
fn test_annotated_move_value_view_vector() {
    // Test vector of primitive types
    let vec_u8 = AnnotatedMoveValue::Vector(
        TypeTag::U8,
        vec![
            AnnotatedMoveValue::U8(1),
            AnnotatedMoveValue::U8(2),
            AnnotatedMoveValue::U8(3),
        ],
    );

    let view: AnnotatedMoveValueView = vec_u8.into();
    let expected = AnnotatedMoveValueView::Vector(vec![
        AnnotatedMoveValueView::U8(1),
        AnnotatedMoveValueView::U8(2),
        AnnotatedMoveValueView::U8(3),
    ]);

    assert_eq!(view, expected);

    let serialized = serde_json::to_value(&view).unwrap();
    let expected_json = json!([1, 2, 3]);
    assert_eq!(serialized, expected_json);

    // Test vector of primitive types
    let vec_u16 = AnnotatedMoveValue::Vector(
        TypeTag::U16,
        vec![
            AnnotatedMoveValue::U16(1),
            AnnotatedMoveValue::U16(u16::MAX),
        ],
    );

    let view: AnnotatedMoveValueView = vec_u16.into();
    let expected = AnnotatedMoveValueView::Vector(vec![
        AnnotatedMoveValueView::U16(1),
        AnnotatedMoveValueView::U16(u16::MAX),
    ]);

    assert_eq!(view, expected);

    let serialized = serde_json::to_value(&view).unwrap();
    let expected_json = json!([1, u16::MAX]);
    //println!("{}", serialized);
    assert_eq!(serialized, expected_json);

    // Test empty vector
    let empty_vec = AnnotatedMoveValue::Vector(TypeTag::U8, vec![]);
    let view: AnnotatedMoveValueView = empty_vec.into();
    let expected = AnnotatedMoveValueView::Vector(vec![]);

    assert_eq!(view, expected);

    let serialized = serde_json::to_value(&view).unwrap();
    let expected_json = json!([]);
    assert_eq!(serialized, expected_json);
}

#[test]
fn test_annotated_move_value_view_struct() {
    // Create a simple struct
    let struct_tag = StructTag {
        address: AccountAddress::from_hex_literal("0x1").unwrap(),
        module: Identifier::new("test").unwrap(),
        name: Identifier::new("TestStruct").unwrap(),
        type_params: vec![],
    };

    let fields = vec![
        (
            Identifier::new("field1").unwrap(),
            AnnotatedMoveValue::U16(u16::MAX),
        ),
        (
            Identifier::new("field2").unwrap(),
            AnnotatedMoveValue::U64(42),
        ),
        (
            Identifier::new("field3").unwrap(),
            AnnotatedMoveValue::Bool(true),
        ),
    ];

    let move_struct = AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: struct_tag.clone(),
        value: fields,
    };

    let input = AnnotatedMoveValue::Struct(move_struct);
    let view: AnnotatedMoveValueView = input.into();

    // Extract struct view
    if let AnnotatedMoveValueView::Struct(struct_view) = &view {
        assert_eq!(struct_view.abilities, 7); // COPY | DROP | STORE = 7
        assert_eq!(struct_view.type_, StrView(struct_tag));

        let fields = &struct_view.value;
        assert_eq!(fields.len(), 3);
        assert_eq!(
            fields.get(&Identifier::new("field1").unwrap()),
            Some(&AnnotatedMoveValueView::U16(u16::MAX))
        );
        assert_eq!(
            fields.get(&Identifier::new("field2").unwrap()),
            Some(&AnnotatedMoveValueView::U64(StrView(42)))
        );
        assert_eq!(
            fields.get(&Identifier::new("field3").unwrap()),
            Some(&AnnotatedMoveValueView::Bool(true))
        );
    } else {
        panic!("Expected Struct variant");
    }

    let serialized = serde_json::to_value(&view).unwrap();
    let expected_json = json!({
        "abilities": 7,
        "type": "0x1::test::TestStruct",
        "value": {
            "field1": u16::MAX,
            "field2": "42",
            "field3": true
        }
    });
    //println!("{}", serialized);
    assert_eq!(serialized, expected_json);
}

#[test]
fn test_annotated_move_value_view_specific_structs_string() {
    // Test MoveString
    let string_value = "Hello, World!";
    let move_string = MoveString::from(string_value);

    let move_string_struct = AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: MoveString::struct_tag(),
        value: vec![(
            Identifier::new("bytes").unwrap(),
            AnnotatedMoveValue::Bytes(move_string.as_bytes().to_vec()),
        )],
    };
    let input = AnnotatedMoveValue::Struct(move_string_struct);
    let view: AnnotatedMoveValueView = input.into();

    if let AnnotatedMoveValueView::SpecificStruct(boxed) = view {
        if let SpecificStructView::MoveString(string) = *boxed {
            assert_eq!(string, move_string);
        } else {
            panic!("Expected MoveString variant");
        }
    } else {
        panic!("Expected SpecificStruct variant");
    }
}

#[test]
fn test_annotated_move_value_view_specific_structs_decimal() {
    // Test DecimalValue
    let decimal_value = DecimalValue::new(1u64.into(), 2);

    let decimal_value_struct = AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: DecimalValue::struct_tag(),
        value: vec![
            (
                Identifier::new("value").unwrap(),
                AnnotatedMoveValue::U256(1u64.into()),
            ),
            (
                Identifier::new("decimal").unwrap(),
                AnnotatedMoveValue::U8(2),
            ),
        ],
    };
    let input = AnnotatedMoveValue::Struct(decimal_value_struct);
    let view: AnnotatedMoveValueView = input.into();

    if let AnnotatedMoveValueView::SpecificStruct(boxed) = view {
        if let SpecificStructView::DecimalValue(value) = *boxed {
            assert_eq!(value, DecimalValueView::from(decimal_value));
        } else {
            panic!("Expected DecimalValue variant");
        }
    } else {
        panic!("Expected SpecificStruct variant");
    }
}

#[test]
fn test_annotated_move_value_view_struct_vector() {
    // Create a vector of structs
    let struct_tag = StructTag {
        address: AccountAddress::from_hex_literal("0x1").unwrap(),
        module: Identifier::new("test").unwrap(),
        name: Identifier::new("TestStruct").unwrap(),
        type_params: vec![],
    };

    let mut structs = Vec::new();

    // Create first struct
    let fields1 = vec![
        (Identifier::new("id").unwrap(), AnnotatedMoveValue::U64(1)),
        (
            Identifier::new("value").unwrap(),
            AnnotatedMoveValue::U64(100),
        ),
    ];

    structs.push(AnnotatedMoveValue::Struct(AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: struct_tag.clone(),
        value: fields1,
    }));

    // Create second struct
    let fields2 = vec![
        (Identifier::new("id").unwrap(), AnnotatedMoveValue::U64(2)),
        (
            Identifier::new("value").unwrap(),
            AnnotatedMoveValue::U64(200),
        ),
    ];

    structs.push(AnnotatedMoveValue::Struct(AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: struct_tag.clone(),
        value: fields2,
    }));

    // Test the vector of structs
    let vector = AnnotatedMoveValue::Vector(TypeTag::Struct(Box::new(struct_tag)), structs);
    let view: AnnotatedMoveValueView = vector.into();

    if let AnnotatedMoveValueView::StructVector(boxed) = view {
        let struct_vector = *boxed;

        assert_eq!(struct_vector.abilities, 7); // COPY | DROP | STORE = 7
        assert!(struct_vector
            .field
            .contains(&StrView(Identifier::new("id").unwrap())));
        assert!(struct_vector
            .field
            .contains(&StrView(Identifier::new("value").unwrap())));
        assert_eq!(struct_vector.value.len(), 2);

        // First struct values
        let first = &struct_vector.value[0];
        assert_eq!(first.len(), 2);
        assert!(first.contains(&AnnotatedMoveValueView::U64(StrView(1))));
        assert!(first.contains(&AnnotatedMoveValueView::U64(StrView(100))));

        // Second struct values
        let second = &struct_vector.value[1];
        assert_eq!(second.len(), 2);
        assert!(second.contains(&AnnotatedMoveValueView::U64(StrView(2))));
        assert!(second.contains(&AnnotatedMoveValueView::U64(StrView(200))));
    } else {
        panic!("Expected StructVector variant");
    }
}

#[test]
fn test_json_serialization() {
    // Test a complex structure with nested values
    let struct_tag = StructTag {
        address: AccountAddress::from_hex_literal("0x1").unwrap(),
        module: Identifier::new("test").unwrap(),
        name: Identifier::new("ComplexStruct").unwrap(),
        type_params: vec![],
    };

    let mut fields = vec![
        (
            Identifier::new("u8_field").unwrap(),
            AnnotatedMoveValue::U8(42),
        ),
        (
            Identifier::new("u64_field").unwrap(),
            AnnotatedMoveValue::U64(1000000),
        ),
        (
            Identifier::new("bool_field").unwrap(),
            AnnotatedMoveValue::Bool(true),
        ),
        (
            Identifier::new("vector_field").unwrap(),
            AnnotatedMoveValue::Vector(
                TypeTag::U8,
                vec![
                    AnnotatedMoveValue::U8(1),
                    AnnotatedMoveValue::U8(2),
                    AnnotatedMoveValue::U8(3),
                ],
            ),
        ),
    ];

    // Add nested struct field
    let nested_fields = vec![(
        Identifier::new("nested_value").unwrap(),
        AnnotatedMoveValue::U64(999),
    )];

    let nested_struct_tag = StructTag {
        address: AccountAddress::from_hex_literal("0x1").unwrap(),
        module: Identifier::new("test").unwrap(),
        name: Identifier::new("NestedStruct").unwrap(),
        type_params: vec![],
    };

    fields.push((
        Identifier::new("nested_struct").unwrap(),
        AnnotatedMoveValue::Struct(AnnotatedMoveStruct {
            abilities: AbilitySet::ALL,
            type_: nested_struct_tag,
            value: nested_fields,
        }),
    ));

    let complex_struct = AnnotatedMoveStruct {
        abilities: AbilitySet::PRIMITIVES,
        type_: struct_tag,
        value: fields,
    };

    let view: AnnotatedMoveValueView = AnnotatedMoveValue::Struct(complex_struct).into();

    // Serialize to JSON
    let json_value = serde_json::to_value(&view).unwrap();

    // Verify JSON structure
    match &json_value {
        Value::Object(obj) => {
            assert!(obj.contains_key("abilities"));
            assert!(obj.contains_key("type"));
            assert!(obj.contains_key("value"));

            if let Value::Object(value_obj) = &obj["value"] {
                assert!(value_obj.contains_key("u8_field"));
                assert_eq!(value_obj["u8_field"], json!(42));

                assert!(value_obj.contains_key("u64_field"));
                assert_eq!(value_obj["u64_field"], json!("1000000"));

                assert!(value_obj.contains_key("bool_field"));
                assert_eq!(value_obj["bool_field"], json!(true));

                assert!(value_obj.contains_key("vector_field"));
                assert_eq!(value_obj["vector_field"], json!([1, 2, 3]));

                assert!(value_obj.contains_key("nested_struct"));
            } else {
                panic!("Expected Object for 'value'");
            }
        }
        _ => panic!("Expected Object for the root JSON value"),
    }
}
