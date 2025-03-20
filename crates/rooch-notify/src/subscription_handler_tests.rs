// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_core_types::value::MoveTypeLayout;
use move_core_types::{ident_str, language_storage::StructTag};
use moveos_types::move_std::string::MoveString;
use moveos_types::state::MoveStructState;
use serde::Deserialize;
use serde::Serialize;

// #[test]
// fn test_to_json_value() {
//     let move_event = TestEvent {
//         creator: AccountAddress::random(),
//         name: "test_event".into(),
//         data: vec![100, 200, 300],
//         // coins: vec![
//         //     RGas::new(ObjectID::random(), 1000000),
//         //     RGas::new(ObjectID::random(), 2000000),
//         //     RGas::new(ObjectID::random(), 3000000),
//         // ],
//     };
//     let event_bytes = bcs::to_bytes(&move_event).unwrap();
//     let sui_move_struct: SuiMoveStruct =
//         BoundedVisitor::deserialize_struct(&event_bytes, &TestEvent::layout())
//             .unwrap()
//             .into();
//     let json_value = sui_move_struct.to_json_value();
//     assert_eq!(
//         Some(&json!("1000000")),
//         json_value.pointer("/coins/0/balance")
//     );
//     assert_eq!(
//         Some(&json!("2000000")),
//         json_value.pointer("/coins/1/balance")
//     );
//     assert_eq!(
//         Some(&json!("3000000")),
//         json_value.pointer("/coins/2/balance")
//     );
//     assert_eq!(
//         Some(&json!(move_event.coins[0].id().to_string())),
//         json_value.pointer("/coins/0/id/id")
//     );
//     assert_eq!(
//         Some(&json!(format!("{:#x}", move_event.creator))),
//         json_value.pointer("/creator")
//     );
//     assert_eq!(Some(&json!("100")), json_value.pointer("/data/0"));
//     assert_eq!(Some(&json!("200")), json_value.pointer("/data/1"));
//     assert_eq!(Some(&json!("300")), json_value.pointer("/data/2"));
//     assert_eq!(Some(&json!("test_event")), json_value.pointer("/name"));
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvent {
    creator: AccountAddress,
    name: MoveString,
    data: Vec<u64>,
}

#[allow(dead_code)]
impl TestEvent {
    fn type_layout() -> StructTag {
        StructTag {
            address: AccountAddress::from_hex_literal("0x42").unwrap(),
            module: ident_str!("test").to_owned(),
            name: ident_str!("test_event").to_owned(),
            type_params: vec![],
        }
    }

    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveTypeLayout::Address,
            MoveTypeLayout::Struct(MoveString::struct_layout()),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U64)),
        ])
    }
}
