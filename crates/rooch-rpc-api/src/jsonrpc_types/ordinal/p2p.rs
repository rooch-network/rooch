// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;

/// Network magic bytes to identify the cryptocurrency network the message was intended for.
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, JsonSchema)]
pub struct MagicView([u8; 4]);
