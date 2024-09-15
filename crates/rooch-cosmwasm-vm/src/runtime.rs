// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::ffi::CString;
use std::ops::Deref;
use std::vec;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use log::{debug, warn};

use once_cell::sync::Lazy;
use serde_json::Value as JSONValue;
use smallvec::{smallvec, SmallVec};
use cosmwasm_std::Checksum;
use cosmwasm_vm::{Cache, CacheOptions, InstanceOptions, Size,capabilities_from_csv };
use super::backend::{MoveBackendApi, MoveStorage, MoveBackendQuerier, build_move_backend};

pub struct WasmVmRuntime<'a> {
  cache: Arc<Cache<MoveBackendApi, MoveStorage<'a>, MoveBackendQuerier>>
}