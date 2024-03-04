// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{account_address::AccountAddress, value::MoveValue};
use moveos_types::moveos_std::{
    context::Context,
    object::{ObjectEntity, RawData},
    object_id::ObjectID,
};

#[derive(Debug, Clone)]
pub enum ObjectArg {
    /// The object argument is &mut Object<T>
    Mutref(ObjectEntity<RawData>),
    /// The object argument is &Object<T>
    Ref(ObjectEntity<RawData>),
    /// The object argument is Object<T>
    Value(ObjectEntity<RawData>),
}

impl ObjectArg {
    pub fn object_id(&self) -> &ObjectID {
        match self {
            ObjectArg::Mutref(object) => &object.id,
            ObjectArg::Ref(object) => &object.id,
            ObjectArg::Value(object) => &object.id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedArg {
    Signer { address: AccountAddress },
    Context { ctx: Context },
    Object(ObjectArg),
    Pure { value: Vec<u8> },
}

impl ResolvedArg {
    pub fn signer(address: AccountAddress) -> Self {
        ResolvedArg::Signer { address }
    }

    pub fn context(ctx: Context) -> Self {
        ResolvedArg::Context { ctx }
    }

    pub fn object_by_mutref(object: ObjectEntity<RawData>) -> Self {
        ResolvedArg::Object(ObjectArg::Mutref(object))
    }

    pub fn object_by_ref(object: ObjectEntity<RawData>) -> Self {
        ResolvedArg::Object(ObjectArg::Ref(object))
    }

    pub fn object_by_value(object: ObjectEntity<RawData>) -> Self {
        ResolvedArg::Object(ObjectArg::Value(object))
    }

    pub fn pure(value: Vec<u8>) -> Self {
        ResolvedArg::Pure { value }
    }

    pub fn into_serialized_arg(self) -> Vec<u8> {
        match self {
            ResolvedArg::Signer { address } => MoveValue::Signer(address)
                .simple_serialize()
                .expect("serialize signer should success"),
            ResolvedArg::Context { ctx } => ctx.to_bytes(),
            ResolvedArg::Object(ObjectArg::Mutref(object)) => object.id.to_bytes(),
            ResolvedArg::Object(ObjectArg::Ref(object)) => object.id.to_bytes(),
            ResolvedArg::Object(ObjectArg::Value(object)) => object.id.to_bytes(),
            ResolvedArg::Pure { value } => value,
        }
    }
}
