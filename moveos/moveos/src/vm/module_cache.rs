// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ambassador::delegate_to_methods;
use bytes::Bytes;
use hashbrown::HashMap;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_binary_format::file_format::CompiledScript;
use move_binary_format::CompiledModule;
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::loader::modules::LegacyModuleCache;
use move_vm_runtime::{Module, RuntimeEnvironment, Script, WithRuntimeEnvironment};
use move_vm_types::code::ambassador_impl_ScriptCache;
use move_vm_types::code::Code;
use move_vm_types::code::{
    ModuleCache, ModuleCode, ModuleCodeBuilder, ScriptCache, UnsyncModuleCache, UnsyncScriptCache,
    WithBytes, WithHash, WithSize,
};
use move_vm_types::sha3_256;
use moveos_store::TxnIndex;
use moveos_types::state_resolver::MoveOSResolver;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanicError {
    CodeInvariantError(String),
}

pub struct Entry<Deserialized, Verified, Extension> {
    /// False if this code is "valid" within the block execution context (i.e., there has been no
    /// republishing of this module so far). If true, executor needs to read the module from the
    /// per-block module caches.
    overridden: AtomicBool,
    /// Cached verified module. Must always be verified.
    module: Arc<ModuleCode<Deserialized, Verified, Extension>>,
}

impl<Deserialized, Verified, Extension> Entry<Deserialized, Verified, Extension>
where
    Verified: Deref<Target = Arc<Deserialized>>,
    Extension: WithSize,
{
    /// Returns a new valid module. Returns a (panic) error if the module is not verified.
    fn new(module: Arc<ModuleCode<Deserialized, Verified, Extension>>) -> Result<Self, PanicError> {
        if !module.code().is_verified() {
            return Err(PanicError::CodeInvariantError(
                "Module code is not verified".to_string(),
            ));
        }

        Ok(Self {
            overridden: AtomicBool::new(false),
            module,
        })
    }

    /// Marks the module as overridden.
    fn mark_overridden(&self) {
        self.overridden.store(true, Ordering::Release)
    }

    /// Returns true if the module is not overridden.
    fn is_not_overridden(&self) -> bool {
        !self.overridden.load(Ordering::Acquire)
    }

    /// Returns the module code stored is this [Entry].
    fn module_code(&self) -> &Arc<ModuleCode<Deserialized, Verified, Extension>> {
        &self.module
    }
}

pub struct GlobalModuleCache<K, D, V, E> {
    /// Module cache containing the verified code.
    pub module_cache: HashMap<K, Entry<D, V, E>>,
    /// Sum of serialized sizes (in bytes) of all cached modules.
    pub size: usize,
}

impl<K, D, V, E> GlobalModuleCache<K, D, V, E>
where
    K: Hash + Eq + Clone,
    V: Deref<Target = Arc<D>>,
    E: WithSize,
{
    /// Returns new empty module cache.
    pub fn empty() -> Self {
        Self {
            module_cache: HashMap::new(),
            size: 0,
        }
    }

    /// Returns true if the key exists in cache and the corresponding module is not overridden.
    pub fn contains_not_overridden(&self, key: &K) -> bool {
        self.module_cache
            .get(key)
            .is_some_and(|entry| entry.is_not_overridden())
    }

    /// Marks the cached module (if it exists) as overridden. As a result, all subsequent calls to
    /// the cache for the associated key will result in a cache miss. If an entry does not exist,
    /// it is a no-op.
    pub fn mark_overridden(&self, key: &K) {
        if let Some(entry) = self.module_cache.get(key) {
            entry.mark_overridden();
        }
    }

    /// Returns the module stored in cache. If the module has not been cached, or it exists but is
    /// overridden, [None] is returned.
    pub fn get(&self, key: &K) -> Option<Arc<ModuleCode<D, V, E>>> {
        self.module_cache.get(key).and_then(|entry| {
            entry
                .is_not_overridden()
                .then(|| Arc::clone(entry.module_code()))
        })
    }

    /// Returns the number of entries in the cache.
    pub fn num_modules(&self) -> usize {
        self.module_cache.len()
    }

    /// Returns the sum of serialized sizes of modules stored in cache.
    pub fn size_in_bytes(&self) -> usize {
        self.size
    }

    /// Flushes the module cache.
    pub fn flush(&mut self) {
        self.module_cache.clear();
        self.size = 0;
    }

    /// Inserts modules into the cache.
    /// Notes:
    ///   1. Only verified modules are inserted.
    ///   2. Not overridden modules should not be removed, and new modules should have unique
    ///      ownership. If these constraints are violated, a panic error is returned.
    pub fn insert_verified(
        &mut self,
        modules: impl Iterator<Item = (K, Arc<ModuleCode<D, V, E>>)>,
    ) -> Result<(), PanicError> {
        use hashbrown::hash_map::Entry::*;

        for (key, module) in modules {
            if let Occupied(entry) = self.module_cache.entry(key.clone()) {
                if entry.get().is_not_overridden() {
                    return Err(PanicError::CodeInvariantError(
                        "Should never replace a non-overridden module".to_string(),
                    ));
                } else {
                    self.size -= entry.get().module_code().extension().size_in_bytes();
                    entry.remove();
                }
            }

            if module.code().is_verified() {
                self.size += module.extension().size_in_bytes();
                let entry =
                    Entry::new(module).expect("Module has been checked and must be verified");
                let prev = self.module_cache.insert(key.clone(), entry);

                // At this point, we must have removed the entry, or returned a panic error.
                assert!(prev.is_none())
            }
        }
        Ok(())
    }

    /// Insert the module to cache. Used for tests only.
    //#[cfg(any(test, feature = "testing"))]
    pub fn insert(&mut self, key: K, module: Arc<ModuleCode<D, V, E>>) {
        self.size += module.extension().size_in_bytes();
        self.module_cache.insert(
            key,
            Entry::new(module).expect("Module code should be verified"),
        );
    }

    /// Removes the module from cache and returns true. If the module does not exist for the
    /// associated key, returns false. Used for tests only.
    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(entry) = self.module_cache.remove(key) {
            self.size -= entry.module_code().extension().size_in_bytes();
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct RoochModuleExtension {
    /// Serialized representation of the module.
    bytes: Bytes,
    /// Module's hash.
    hash: [u8; 32],
    /// The state value metadata associated with the module, when read from or
    /// written to storage.
    state_value_metadata: StateValueMetadata,
}

impl RoochModuleExtension {
    /// Creates new extension based on [StateValue].
    pub fn new(state_value: StateValue) -> Self {
        let (state_value_metadata, bytes) = state_value.unpack();
        let hash = sha3_256(&bytes);
        Self {
            bytes,
            hash,
            state_value_metadata,
        }
    }

    /// Returns the state value metadata stored in extension.
    pub fn state_value_metadata(&self) -> &StateValueMetadata {
        &self.state_value_metadata
    }
}

impl WithBytes for RoochModuleExtension {
    fn bytes(&self) -> &Bytes {
        &self.bytes
    }
}

impl WithHash for RoochModuleExtension {
    fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct StateValueMetadataInner {
    slot_deposit: u64,
    bytes_deposit: u64,
    creation_time_usecs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StateValueMetadata {
    inner: Option<StateValueMetadataInner>,
}

impl StateValueMetadata {
    pub fn into_persistable(self) -> Option<PersistedStateValueMetadata> {
        self.inner.map(|inner| {
            let StateValueMetadataInner {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            } = inner;
            if bytes_deposit == 0 {
                PersistedStateValueMetadata::V0 {
                    deposit: slot_deposit,
                    creation_time_usecs,
                }
            } else {
                PersistedStateValueMetadata::V1 {
                    slot_deposit,
                    bytes_deposit,
                    creation_time_usecs,
                }
            }
        })
    }

    pub fn new(
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: &CurrentTimeMicroseconds,
    ) -> Self {
        Self::new_impl(
            slot_deposit,
            bytes_deposit,
            creation_time_usecs.microseconds,
        )
    }

    pub fn legacy(slot_deposit: u64, creation_time_usecs: &CurrentTimeMicroseconds) -> Self {
        Self::new(slot_deposit, 0, creation_time_usecs)
    }

    pub fn placeholder(creation_time_usecs: &CurrentTimeMicroseconds) -> Self {
        Self::legacy(0, creation_time_usecs)
    }

    pub fn none() -> Self {
        Self { inner: None }
    }

    fn new_impl(slot_deposit: u64, bytes_deposit: u64, creation_time_usecs: u64) -> Self {
        Self {
            inner: Some(StateValueMetadataInner {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            }),
        }
    }

    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    fn inner(&self) -> Option<&StateValueMetadataInner> {
        self.inner.as_ref()
    }

    pub fn creation_time_usecs(&self) -> u64 {
        self.inner().map_or(0, |v1| v1.creation_time_usecs)
    }

    pub fn slot_deposit(&self) -> u64 {
        self.inner().map_or(0, |v1| v1.slot_deposit)
    }

    pub fn bytes_deposit(&self) -> u64 {
        self.inner().map_or(0, |v1| v1.bytes_deposit)
    }

    pub fn total_deposit(&self) -> u64 {
        self.slot_deposit() + self.bytes_deposit()
    }

    pub fn maybe_upgrade(&mut self) -> &mut Self {
        *self = Self::new_impl(
            self.slot_deposit(),
            self.bytes_deposit(),
            self.creation_time_usecs(),
        );
        self
    }

    fn expect_upgraded(&mut self) -> &mut StateValueMetadataInner {
        self.inner.as_mut().expect("State metadata is None.")
    }

    pub fn set_slot_deposit(&mut self, amount: u64) {
        self.expect_upgraded().slot_deposit = amount;
    }

    pub fn set_bytes_deposit(&mut self, amount: u64) {
        self.expect_upgraded().bytes_deposit = amount;
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "StateValueMetadata")]
pub enum PersistedStateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}

impl PersistedStateValueMetadata {
    pub fn into_in_mem_form(self) -> StateValueMetadata {
        match self {
            PersistedStateValueMetadata::V0 {
                deposit,
                creation_time_usecs,
            } => StateValueMetadata::new_impl(deposit, 0, creation_time_usecs),
            PersistedStateValueMetadata::V1 {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            } => StateValueMetadata::new_impl(slot_deposit, bytes_deposit, creation_time_usecs),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct CurrentTimeMicroseconds {
    pub microseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateValue {
    data: Bytes,
    metadata: StateValueMetadata,
}

#[allow(dead_code)]
impl StateValue {
    fn to_perishable_form(&self) -> PersistedStateValue {
        let Self { data, metadata } = self.clone();
        let metadata = metadata.into_persistable();
        match metadata {
            None => PersistedStateValue::V0(data),
            Some(metadata) => PersistedStateValue::WithMetadata { data, metadata },
        }
    }
}

impl StateValue {
    pub fn new_legacy(bytes: Bytes) -> Self {
        Self::new_with_metadata(bytes, StateValueMetadata::none())
    }

    pub fn new_with_metadata(data: Bytes, metadata: StateValueMetadata) -> Self {
        Self { data, metadata }
    }

    pub fn size(&self) -> usize {
        self.bytes().len()
    }

    pub fn bytes(&self) -> &Bytes {
        &self.data
    }

    /// Applies a bytes-to-bytes transformation on the state value contents,
    /// leaving the state value metadata untouched.
    pub fn map_bytes<F: FnOnce(Bytes) -> anyhow::Result<Bytes>>(
        self,
        f: F,
    ) -> anyhow::Result<StateValue> {
        Ok(Self::new_with_metadata(f(self.data)?, self.metadata))
    }

    pub fn into_bytes(self) -> Bytes {
        self.data
    }

    pub fn set_bytes(&mut self, data: Bytes) {
        self.data = data;
    }

    pub fn metadata(&self) -> &StateValueMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut StateValueMetadata {
        &mut self.metadata
    }

    pub fn into_metadata(self) -> StateValueMetadata {
        self.metadata
    }

    pub fn unpack(self) -> (StateValueMetadata, Bytes) {
        let Self { data, metadata } = self;
        (metadata, data)
    }
}

// #[cfg(any(test, feature = "fuzzing"))]
impl From<Vec<u8>> for StateValue {
    fn from(bytes: Vec<u8>) -> Self {
        StateValue::new_legacy(bytes.into())
    }
}

impl From<Bytes> for StateValue {
    fn from(bytes: Bytes) -> Self {
        StateValue::new_legacy(bytes)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "StateValue")]
enum PersistedStateValue {
    V0(Bytes),
    WithMetadata {
        data: Bytes,
        metadata: PersistedStateValueMetadata,
    },
}

#[derive(Clone)]
pub struct MoveOSCodeCache<'a, S> {
    pub runtime_environment: &'a RuntimeEnvironment,
    pub script_cache: UnsyncScriptCache<[u8; 32], CompiledScript, Script>,
    pub module_cache:
        UnsyncModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension, Option<TxnIndex>>,
    pub global_module_cache:
        Arc<RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>>,
    pub legacy_module_cache: LegacyModuleCache,
    pub resolver: &'a S,
}

impl<'a, S: MoveOSResolver> WithRuntimeEnvironment for MoveOSCodeCache<'a, S> {
    fn runtime_environment(&self) -> &RuntimeEnvironment {
        self.runtime_environment
    }
}

impl<'a, S> MoveOSCodeCache<'a, S> {
    pub fn new(
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'a RuntimeEnvironment,
        resolver: &'a S,
    ) -> Self {
        Self {
            script_cache: UnsyncScriptCache::empty(),
            module_cache: UnsyncModuleCache::empty(),
            global_module_cache: global_module_cache.clone(),
            runtime_environment,
            legacy_module_cache: LegacyModuleCache::new(),
            resolver,
        }
    }

    pub fn get_script_cache(&self) -> &UnsyncScriptCache<[u8; 32], CompiledScript, Script> {
        &self.script_cache
    }

    pub fn get_module_cache(
        &self,
    ) -> &dyn ModuleCache<
        Key = ModuleId,
        Deserialized = CompiledModule,
        Verified = Module,
        Extension = RoochModuleExtension,
        Version = Option<TxnIndex>,
    > {
        &self.module_cache
    }
}

#[delegate_to_methods]
#[delegate(ScriptCache, target_ref = "as_script_cache")]
impl<'a, S> MoveOSCodeCache<'a, S> {
    pub fn as_script_cache(
        &self,
    ) -> &dyn ScriptCache<Key = [u8; 32], Deserialized = CompiledScript, Verified = Script> {
        self.get_script_cache()
    }

    fn as_module_cache(
        &self,
    ) -> &dyn ModuleCache<
        Key = ModuleId,
        Deserialized = CompiledModule,
        Verified = Module,
        Extension = RoochModuleExtension,
        Version = Option<TxnIndex>,
    > {
        self.get_module_cache()
    }
}

impl<'a, S> ModuleCache for MoveOSCodeCache<'a, S> {
    type Key = ModuleId;
    type Deserialized = CompiledModule;
    type Verified = Module;
    type Extension = RoochModuleExtension;
    type Version = Option<TxnIndex>;

    fn insert_deserialized_module(
        &self,
        key: Self::Key,
        deserialized_code: Self::Deserialized,
        extension: Arc<Self::Extension>,
        version: Self::Version,
    ) -> VMResult<()> {
        self.module_cache
            .insert_deserialized_module(key, deserialized_code, extension, version)
    }

    fn insert_verified_module(
        &self,
        key: Self::Key,
        verified_code: Self::Verified,
        extension: Arc<Self::Extension>,
        version: Self::Version,
    ) -> VMResult<Arc<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension>>> {
        // insert verified code to the global module cache
        let mut write_guard = self.global_module_cache.write();
        let m = Arc::new(ModuleCode::from_verified(
            verified_code.clone(),
            extension.clone(),
        ));
        write_guard.insert(key.clone(), m.clone());
        // insert verified code to the module cache
        self.module_cache
            .insert_verified_module(key.clone(), verified_code, extension, version)
    }

    fn get_module_or_build_with(
        &self,
        key: &Self::Key,
        builder: &dyn ModuleCodeBuilder<
            Key = Self::Key,
            Deserialized = Self::Deserialized,
            Verified = Self::Verified,
            Extension = Self::Extension,
        >,
    ) -> VMResult<
        Option<(
            Arc<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension>>,
            Self::Version,
        )>,
    > {
        let read_guard = self.global_module_cache.read();

        if let Some(module) = read_guard.get(key) {
            return Ok(Some((module, Self::Version::default())));
        }

        let read = self.module_cache.get_module_or_build_with(key, builder)?;
        Ok(read)
    }

    fn num_modules(&self) -> usize {
        self.as_module_cache().num_modules()
    }
}

impl<'a, S: MoveOSResolver> ModuleCodeBuilder for MoveOSCodeCache<'a, S> {
    type Key = ModuleId;
    type Deserialized = CompiledModule;
    type Verified = Module;
    type Extension = RoochModuleExtension;

    fn build(
        &self,
        key: &Self::Key,
    ) -> VMResult<Option<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension>>> {
        let module_bytes = match self.resolver.get_module(key) {
            Err(e) => return Err(e.finish(Location::Module(key.clone()))),
            Ok(module_opt) => match module_opt {
                None => {
                    return Err(PartialVMError::new(StatusCode::RESOURCE_DOES_NOT_EXIST)
                        .finish(Location::Module(key.clone())))
                }
                Some(bytes) => bytes,
            },
        };

        let compiled_module = self
            .runtime_environment()
            .deserialize_into_compiled_module(&module_bytes)?;

        let extension = Arc::new(RoochModuleExtension::new(StateValue::new_legacy(
            Bytes::copy_from_slice(module_bytes.as_ref()),
        )));

        let module = ModuleCode::from_deserialized(compiled_module, extension);

        Ok(Some(module))
    }
}
