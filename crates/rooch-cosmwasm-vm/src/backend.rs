// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};
use cosmwasm_std::{Order, Record};
use cosmwasm_vm::{Backend, Storage, BackendApi, Querier, BackendResult, GasInfo, BackendError};
use move_core_types::value::MoveStructLayout;
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::StatusCode;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::values::Value;
use moveos_object_runtime::runtime_object::RuntimeObject;
use moveos_object_runtime::TypeLayoutLoader;
use moveos_types::state_resolver::StatelessResolver;

use moveos_types::state::FieldKey;
use moveos_types::state::MoveState;
use moveos_types::h256;
use moveos_types::state::MoveType;
use std::collections::{BTreeMap, HashMap};

/*
pub struct MoveBackend {
    object: Arc<Mutex<RuntimeObject>>,
    type_mapping: Arc<Mutex<BTreeMap<Vec<u8>, MoveType>>>,
    iterators: Mutex<HashMap<u32, Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + Send>>>,
    next_iterator_id: Mutex<u32>,
}

impl MoveBackend {
    pub fn new() -> Self {
        MoveBackend {
            object: Arc::new(Mutex::new(RuntimeObject::new())),
            type_mapping: Arc::new(Mutex::new(BTreeMap::new())),
            iterators: Mutex::new(HashMap::new()),
            next_iterator_id: Mutex::new(0),
        }
    }

    fn get_move_type_from_key(&self, key: &[u8]) -> Result<MoveType, BackendError> {
        let type_mapping = self.type_mapping.lock().unwrap();
        type_mapping.get(key).cloned().ok_or_else(|| {
            BackendError::Unknown {
                msg: format!("No MoveType found for key: {:?}", key),
            }
        })
    }

    fn serialize_value(&self, value: &Value) -> Result<Vec<u8>, BackendError> {
        // Implement serialization logic
        Ok(vec![]) // Placeholder
    }

    fn deserialize_value(&self, move_type: &MoveType, value: &[u8]) -> Result<Value, BackendError> {
        // Implement deserialization logic
        Ok(Value::u8(0)) // Placeholder
    }

    pub fn into_backend(self) -> Backend<BackendApi, Storage, Querier> {
        Backend {
            api: MoveBackendApi,
            storage: self,
            querier: MoveBackendQuerier,
        }
    }
}
 */

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

    fn serialize_value(&self, layout: &MoveTypeLayout, value: &Value) -> Result<Vec<u8>, BackendError> {
        let bytes = match value.simple_serialize(layout) {
            Some(bytes) => bytes,
            None => return Err(BackendError::BadArgument{}),
        };
        Ok(bytes)
    }

    fn deserialize_value(&self, layout: &MoveTypeLayout, bytes: &[u8]) -> Result<Value, BackendError> {
        let value = match Value::simple_deserialize(bytes, layout) {
            Some(value) => value,
            None => return Err(BackendError::BadArgument{}),
        };
        Ok(value)
    }
}

impl<'a> Storage for MoveStorage<'a>  {
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
                    (Err(BackendError::Unknown { msg: e.to_string() }), GasInfo::new(1, 0))
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

        match object.add_field(self.layout_loader, self.resolver, field_key, &move_type, deserialized_value) {
            Ok(_) => (Ok(()), GasInfo::new(1, 0)),
            Err(e) => (Err(BackendError::Unknown { msg: e.to_string() }), GasInfo::new(1, 0)),
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
                    (Err(BackendError::Unknown { msg: e.to_string() }), GasInfo::new(1, 0))
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

        let (values, _bytes_len_opt) = object.scan_fields(
            self.layout_loader,
            self.resolver,
            cursor,
            usize::MAX,
            &Type::Vector(Box::new(Type::U8)), // Assuming values are Vec<u8>
        )
        .map_err(|e| BackendError::Unknown { msg: e.to_string() })?;

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

        (Ok(id), GasInfo::new(1, 0))
    }

    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        let (records, index) = match self.iterators.get_mut(&iterator_id) {
            Some(it) => it,
            None => return (Err(BackendError::IteratorDoesNotExist {id: iterator_id}), GasInfo::new(1, 0)),
        };

        if *index >= records.len() {
            return (Ok(None), GasInfo::new(1, 0));
        }

        let (key, value) = &records[*index];
        *index += 1;

        let key_bytes: [u8; 32] = key.0.into();

        (Ok(Some((key_bytes.to_vec(), value.clone()))), GasInfo::new(1, 0))
    }
}

/* 
// Implement BackendApi
pub struct MoveBackendApi;

impl BackendApi for MoveBackendApi {
    fn addr_validate(&self, human: &str) -> BackendResult<()> {
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
pub struct MoveBackendQuerier;

impl Querier for MoveBackendQuerier {
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        // Implement query logic
        (
            Ok(SystemResult::Ok(ContractResult::Ok(Binary::from(vec![])))),
            GasInfo::with_externally_used(gas_limit),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_existing_key() {
        let mut backend = MoveBackend::new();

        // Prepare test data
        let key = b"test_key".to_vec();
        let value = Value::u8(42);
        let move_type = MoveType::U8;

        // Add test data to the backend
        {
            let mut type_mapping = backend.type_mapping.lock().unwrap();
            type_mapping.insert(key.clone(), move_type.clone());
        }
        {
            let mut obj = backend.object.lock().unwrap();
            obj.add_field(FieldKey::from_bytes(&key), &move_type, value.clone()).unwrap();
        }

        // Test get method
        let (result, _) = backend.get(&key);
        assert!(result.is_ok());
        let retrieved_value = result.unwrap();
        assert!(retrieved_value.is_some());
        // Note: This assertion might need to be adjusted based on your actual serialization logic
        assert_eq!(retrieved_value.unwrap(), vec![]);
    }
}*/