// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::{IndexedFieldState, IndexedObjectState};
use anyhow::Result;
use move_core_types::language_storage::TypeTag;
use move_core_types::resolver::MoveResolver;
use move_resource_viewer::MoveValueAnnotator;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{KeyState, MoveStructType, State};
use rooch_rpc_api::jsonrpc_types::AnnotatedMoveValueView;
use rooch_types::bitcoin::utxo::UTXO;

pub mod indexer;
pub mod messages;
pub mod reader_indexer;

pub fn resolve_state_to_json(
    resolver: &dyn MoveResolver,
    ty_tag: &TypeTag,
    value: &[u8],
) -> Result<String> {
    let annotator_state = MoveValueAnnotator::new(&resolver).view_value(ty_tag, value)?;
    let annotator_state_view = AnnotatedMoveValueView::from(annotator_state);
    let annotator_state_json = serde_json::to_string(&annotator_state_view)?;
    Ok(annotator_state_json)
}

pub fn is_utxo_object(state_opt: Option<State>) -> bool {
    match state_opt {
        Some(state) => state.match_struct_type(&UTXO::struct_tag()),
        None => false,
    }
}

pub fn new_object_state_from_raw_object(
    value: State,
    tx_order: u64,
    state_index: u64,
) -> Result<IndexedObjectState> {
    let raw_object = value.as_raw_object()?;

    let state = IndexedObjectState::new_from_raw_object(raw_object, tx_order, state_index);
    Ok(state)
}

pub fn new_field_state(
    key: KeyState,
    value: State,
    object_id: ObjectID,
    tx_order: u64,
    state_index: u64,
) -> Result<IndexedFieldState> {
    let key_hex = key.to_string();
    let state = IndexedFieldState::new(
        object_id,
        key_hex,
        key.key_type,
        value.value_type,
        tx_order,
        state_index,
    );
    Ok(state)
}
