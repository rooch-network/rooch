// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use std::str::FromStr;

use crate::jsonrpc_types::*;

fn str_view_test_round_trip<T>(view: StrView<T>, expect_str: &str)
where
    T: PartialEq + std::fmt::Debug,
    StrView<T>: ToString + std::str::FromStr + PartialEq + std::fmt::Display + std::fmt::Debug,
    <StrView<T> as std::str::FromStr>::Err: std::fmt::Debug,
{
    let s = view.to_string();
    assert_eq!(
        s, expect_str,
        "String mismatch: expected {}, got {}",
        expect_str, s
    );
    let view2 = StrView::<T>::from_str(&s).unwrap_or_else(|e| {
        panic!("from_str failed: {:?}", e);
    });
    assert_eq!(
        view, view2,
        "View mismatch: expected {:?}, got {:?}",
        view, view2
    );
    assert_eq!(
        view.0, view2.0,
        "Value mismatch: expected {:?}, got {:?}",
        view.0, view2.0
    );
}

#[test]
fn test_str_view() {
    str_view_test_round_trip(StrView(1u64), "1");
    str_view_test_round_trip(StrView(1i64), "1");
    str_view_test_round_trip(StrView(1u128), "1");
    str_view_test_round_trip(StrView(1i128), "1");

    str_view_test_round_trip(StrView(move_core_types::u256::U256::from(1u64)), "1");

    str_view_test_round_trip(StrView(ethers::types::U64::from(1u64)), "0x1");
    str_view_test_round_trip(StrView(ethers::types::U256::from(1u64)), "0x1");

    str_view_test_round_trip(
        StrView(ethers::types::H64::from_str("0x0000000000000001").unwrap()),
        "0x0000000000000001",
    );
    str_view_test_round_trip(
        StrView(
            ethers::types::H160::from_str("0x0000000000000000000000000000000000000001").unwrap(),
        ),
        "0x0000000000000000000000000000000000000001",
    );
    str_view_test_round_trip(
        StrView(
            ethers::types::H256::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
        ),
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    );
}

#[test]
fn test_bytes_serialize() {
    let bytes = BytesView::from_str("0123456789abcdef").unwrap();
    let serialized = serde_json::to_string(&bytes).unwrap();
    assert_eq!(serialized, r#""0x0123456789abcdef""#);
}

#[test]
fn test_bytes_deserialize() {
    let bytes0: Result<BytesView, serde_json::Error> = serde_json::from_str(r#""∀∂""#);
    let bytes1: Result<BytesView, serde_json::Error> = serde_json::from_str(r#""""#);
    let bytes2: Result<BytesView, serde_json::Error> = serde_json::from_str(r#""0x123""#);
    let bytes3: Result<BytesView, serde_json::Error> = serde_json::from_str(r#""0xgg""#);

    let bytes4: BytesView = serde_json::from_str(r#""0x""#).unwrap();
    let bytes5: BytesView = serde_json::from_str(r#""0x12""#).unwrap();
    let bytes6: BytesView = serde_json::from_str(r#""0x0123""#).unwrap();
    let bytes7: BytesView = serde_json::from_str(r#""0123""#).unwrap();

    assert!(bytes0.is_err());
    assert_eq!(bytes1.unwrap(), BytesView::from(vec![]));
    assert!(bytes2.is_err());
    assert!(bytes3.is_err());
    assert_eq!(bytes4, BytesView::from(vec![]));
    assert_eq!(bytes5, BytesView::from(vec![0x12]));
    assert_eq!(bytes6, BytesView::from(vec![0x1, 0x23]));
    assert_eq!(bytes7, BytesView::from(vec![0x1, 0x23]));
}

#[test]
fn test_account_address_view() {
    str_view_test_round_trip(
        AccountAddressView::from(AccountAddress::ONE),
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    );
    str_view_test_round_trip(
        AccountAddressView::from_str("0x1").unwrap(),
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    );
    str_view_test_round_trip(
        AccountAddressView::from_str(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap(),
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    );

    let address_result = AccountAddressView::from_str("11");
    assert!(address_result.is_err());
}
