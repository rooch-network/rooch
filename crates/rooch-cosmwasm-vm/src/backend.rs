// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cosmwasm_std::{Binary, ContractResult, Order, Record, SystemResult};
use cosmwasm_vm::{Backend, BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage};

use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::StatusCode;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::values::Value;

use moveos_object_runtime::runtime_object::RuntimeObject;
use moveos_object_runtime::TypeLayoutLoader;
use moveos_types::h256;
use moveos_types::state::FieldKey;
use moveos_types::state::MoveState;
use moveos_types::state_resolver::StatelessResolver;

pub struct MoveStorage<'a> {
    object: Arc<Mutex<RuntimeObject>>,
    layout_loader: &'a dyn TypeLayoutLoader,
    resolver: &'a dyn StatelessResolver,
    iterator_id_counter: u32,
    iterators: HashMap<u32, (Vec<(FieldKey, Vec<u8>)>, usize)>,
}

impl<'a> MoveStorage<'a> {
    pub fn new(
        object: Arc<Mutex<RuntimeObject>>,
        layout_loader: &'a dyn TypeLayoutLoader,
        resolver: &'a dyn StatelessResolver,
    ) -> Self {
        MoveStorage {
            object,
            layout_loader,
            resolver,
            iterator_id_counter: 0,
            iterators: HashMap::new(),
        }
    }

    fn serialize_value(
        &self,
        layout: &MoveTypeLayout,
        value: &Value,
    ) -> Result<Vec<u8>, BackendError> {
        let bytes = match value.simple_serialize(layout) {
            Some(bytes) => bytes,
            None => return Err(BackendError::BadArgument {}),
        };
        Ok(bytes)
    }

    fn deserialize_value(
        &self,
        layout: &MoveTypeLayout,
        bytes: &[u8],
    ) -> Result<Value, BackendError> {
        let value = match Value::simple_deserialize(bytes, layout) {
            Some(value) => value,
            None => return Err(BackendError::BadArgument {}),
        };
        Ok(value)
    }
}

impl<'a> Storage for MoveStorage<'a> {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        let object = self.object.lock().unwrap();

        let hash = h256::sha3_256_of(key);
        let field_key = FieldKey::new(hash.into());
        let move_layout = Vec::<u8>::type_layout();
        let move_type = Type::Vector(Box::new(Type::U8));

        match object.get_field(self.layout_loader, self.resolver, field_key, &move_type) {
            Ok(value) => match self.serialize_value(&move_layout, &value.0) {
                Ok(bytes) => (Ok(Some(bytes)), GasInfo::new(1, 0)),
                Err(e) => (Err(e), GasInfo::new(1, 0)),
            },
            Err(e) => {
                if e.major_status() == StatusCode::RESOURCE_DOES_NOT_EXIST {
                    (Ok(None), GasInfo::new(1, 0))
                } else {
                    (
                        Err(BackendError::Unknown { msg: e.to_string() }),
                        GasInfo::new(1, 0),
                    )
                }
            }
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        let mut object = self.object.lock().unwrap();
        let hash = h256::sha3_256_of(key);
        let field_key = FieldKey::new(hash.into());
        let move_layout = Vec::<u8>::type_layout();
        let move_type = Type::Vector(Box::new(Type::U8));

        let deserialized_value = match self.deserialize_value(&move_layout, value) {
            Ok(v) => v,
            Err(e) => return (Err(e), GasInfo::new(1, 0)),
        };

        match object.add_field(
            self.layout_loader,
            self.resolver,
            field_key,
            &move_type,
            deserialized_value,
        ) {
            Ok(_) => (Ok(()), GasInfo::new(1, 0)),
            Err(e) => (
                Err(BackendError::Unknown { msg: e.to_string() }),
                GasInfo::new(1, 0),
            ),
        }
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        let mut object = self.object.lock().unwrap();
        let hash = h256::sha3_256_of(key);
        let field_key = FieldKey::new(hash.into());
        let move_type = Type::Vector(Box::new(Type::U8));

        match object.remove_field(self.layout_loader, self.resolver, field_key, &move_type) {
            Ok(_) => (Ok(()), GasInfo::new(1, 0)),
            Err(e) => {
                if e.major_status() == StatusCode::RESOURCE_DOES_NOT_EXIST {
                    (Ok(()), GasInfo::new(1, 0))
                } else {
                    (
                        Err(BackendError::Unknown { msg: e.to_string() }),
                        GasInfo::new(1, 0),
                    )
                }
            }
        }
    }

    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        let object = self.object.lock().unwrap();
        let cursor = start.map(|s| FieldKey::new(h256::sha3_256_of(s).into()));

        match object.scan_fields(
            self.layout_loader,
            self.resolver,
            cursor,
            usize::MAX,
            &Type::Vector(Box::new(Type::U8)), // Assuming values are Vec<u8>
        ) {
            Ok((values, bytes_len_opt)) => {
                let move_layout = Vec::<u8>::type_layout();

                let mut records: Vec<(FieldKey, Vec<u8>)> = values
                    .into_iter()
                    .filter_map(|(key, value)| {
                        self.serialize_value(&move_layout, &value)
                            .ok()
                            .map(|bytes| (key, bytes))
                    })
                    .collect();

                if order == Order::Descending {
                    records.reverse();
                }

                // Apply end filter
                if let Some(end_bytes) = end {
                    let end_key = FieldKey::new(h256::sha3_256_of(end_bytes).into());
                    records.retain(|(key, _)| match order {
                        Order::Ascending => *key < end_key,
                        Order::Descending => *key > end_key,
                    });
                }

                let id = self.iterator_id_counter;
                self.iterator_id_counter += 1;
                self.iterators.insert(id, (records, 0));

                let gas_used = if let Some(bytes_len) = bytes_len_opt {
                    bytes_len.into()
                } else {
                    0
                };
                (Ok(id), GasInfo::new(gas_used, 0))
            }
            Err(e) => (
                Err(BackendError::Unknown { msg: e.to_string() }),
                GasInfo::new(1, 0),
            ),
        }
    }

    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        let (records, index) = match self.iterators.get_mut(&iterator_id) {
            Some(it) => it,
            None => {
                return (
                    Err(BackendError::IteratorDoesNotExist { id: iterator_id }),
                    GasInfo::new(1, 0),
                )
            }
        };

        if *index >= records.len() {
            return (Ok(None), GasInfo::new(1, 0));
        }

        let (key, value) = &records[*index];
        *index += 1;

        let key_bytes: [u8; 32] = key.0.into();

        (
            Ok(Some((key_bytes.to_vec(), value.clone()))),
            GasInfo::new(1, 0),
        )
    }
}

// Implement BackendApi
#[derive(Clone)]
pub struct MoveBackendApi;

impl BackendApi for MoveBackendApi {
    fn addr_validate(&self, _human: &str) -> BackendResult<()> {
        // Implement address validation logic
        (Ok(()), GasInfo::new(1, 0))
    }

    fn addr_canonicalize(&self, human: &str) -> BackendResult<Vec<u8>> {
        // Implement address canonicalization logic
        (Ok(human.as_bytes().to_vec()), GasInfo::new(1, 0))
    }

    fn addr_humanize(&self, canonical: &[u8]) -> BackendResult<String> {
        // Implement address humanization logic
        match String::from_utf8(canonical.to_vec()) {
            Ok(human) => (Ok(human), GasInfo::new(1, 0)),
            Err(_) => (Err(BackendError::InvalidUtf8 {}), GasInfo::new(1, 0)),
        }
    }
}

// Implement Querier
#[derive(Clone)]
pub struct MoveBackendQuerier;

impl Querier for MoveBackendQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        // Implement query logic
        (
            Ok(SystemResult::Ok(ContractResult::Ok(Binary::from(vec![])))),
            GasInfo::with_externally_used(gas_limit),
        )
    }
}

pub fn build_move_backend<'a>(
    object: Arc<Mutex<RuntimeObject>>,
    layout_loader: &'a dyn TypeLayoutLoader,
    resolver: &'a dyn StatelessResolver,
) -> Backend<MoveBackendApi, MoveStorage<'a>, MoveBackendQuerier> {
    Backend {
        api: MoveBackendApi,
        storage: MoveStorage::new(object, layout_loader, resolver),
        querier: MoveBackendQuerier,
    }
}
