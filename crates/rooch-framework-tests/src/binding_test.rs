// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos::moveos::MoveOS;
use moveos_store::MoveOSDB;
use moveos_types::module_binding::{ModuleBundle, MoveFunctionCaller};
use rooch_genesis::RoochGenesis;

pub struct RustBindingTest {
    moveos: MoveOS,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let moveosdb = MoveOSDB::new_with_memory_store();
        let genesis: &RoochGenesis = &rooch_genesis::ROOCH_GENESIS;

        let mut moveos = MoveOS::new(moveosdb, genesis.all_natives(), genesis.config.clone())?;
        if moveos.state().is_genesis() {
            moveos.init_genesis(genesis.genesis_txs())?;
        }
        Ok(Self { moveos })
    }

    pub fn as_module_bundle<'a, M: ModuleBundle<'a>>(&'a self) -> M {
        self.moveos.as_module_bundle::<M>()
    }
}
