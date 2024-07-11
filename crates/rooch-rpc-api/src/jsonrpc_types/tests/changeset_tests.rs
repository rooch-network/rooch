// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ::rooch_types::test_utils::random_state_change_set;

use crate::jsonrpc_types::*;

#[test]
fn test_changeset() {
    let changeset = random_state_change_set();
    let changeset_view = StateChangeSetView::from(changeset);
    let json = serde_json::to_string_pretty(&changeset_view).unwrap();
    //println!("{}", json);
    let changeset_view2: StateChangeSetView = serde_json::from_str(&json).unwrap();
    assert_eq!(changeset_view, changeset_view2);
}
