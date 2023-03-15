// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::MoveResolverExt;
use framework::natives::{self, GasParameters};
use move_binary_format::errors::VMResult;
use move_bytecode_verifier::VerifierConfig;
use move_table_extension::NativeTableContext;
use move_vm_runtime::{
    config::VMConfig, move_vm::MoveVM, native_extensions::NativeContextExtensions, session::Session,
};
use statedb::HashValue;

pub struct MoveVmExt {
    inner: MoveVM,
}

impl MoveVmExt {
    pub fn new() -> VMResult<Self> {
        let gas_params = GasParameters::zeros();
        Ok(Self {
            inner: MoveVM::new_with_config(
                natives::all_natives(gas_params),
                VMConfig {
                    verifier: VerifierConfig::default(),
                    max_binary_format_version: 6,
                    paranoid_type_checks: false,
                },
            )?,
        })
    }

    pub fn new_session<'r, S: MoveResolverExt>(
        &self,
        remote: &'r S,
        session_id: HashValue,
    ) -> Session<'r, '_, S> {
        let mut extensions = NativeContextExtensions::default();
        let txn_hash: [u8; 32] = session_id.into();

        extensions.add(NativeTableContext::new(txn_hash, remote));

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        self.inner.flush_loader_cache_if_invalidated();

        self.inner.new_session_with_extensions(remote, extensions)
    }
}
