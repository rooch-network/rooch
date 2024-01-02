// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{ExecuteTransactionMessage, ExecuteTransactionResult};
use accumulator::inmemory::InMemoryAccumulator;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos::moveos::MoveOS;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use rooch_genesis::RoochGenesis;
use rooch_store::RoochStore;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};
use rooch_types::transaction::AbstractTransaction;

pub struct ExecutorActor {
    genesis: RoochGenesis,
    moveos: MoveOS,
    rooch_store: RoochStore,
}

impl ExecutorActor {
    pub fn new(
        genesis_ctx: GenesisContext,
        bitcoin_genesis_ctx: BitcoinGenesisContext,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let genesis: RoochGenesis =
            rooch_genesis::RoochGenesis::build(genesis_ctx, bitcoin_genesis_ctx)?;
        let moveos = MoveOS::new(
            moveos_store,
            genesis.all_natives(),
            genesis.config.clone(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
        )?;

        let executor = Self {
            genesis,
            moveos,
            rooch_store,
        };
        executor.init_or_check_genesis()
    }

    fn init_or_check_genesis(mut self) -> Result<Self> {
        if self.moveos.state().is_genesis() {
            let genesis_result = self.moveos.init_genesis(
                self.genesis.genesis_txs(),
                self.genesis.genesis_ctx(),
                self.genesis.bitcoin_genesis_ctx(),
            )?;
            let genesis_state_root = genesis_result
                .last()
                .expect("Genesis result must not empty")
                .0;

            //TODO should we save the genesis txs to sequencer?
            for (genesis_tx, (state_root, genesis_tx_output)) in
                self.genesis.genesis_txs().into_iter().zip(genesis_result)
            {
                let tx_hash = genesis_tx.tx_hash();
                self.handle_tx_output(tx_hash, state_root, genesis_tx_output)?;
            }

            debug_assert!(
                genesis_state_root == self.genesis.genesis_state_root(),
                "Genesis state root mismatch"
            );
            let genesis_info =
                GenesisInfo::new(self.genesis.genesis_package_hash(), genesis_state_root);
            self.moveos.config_store().save_genesis(genesis_info)?;
        } else {
            self.genesis.check_genesis(self.moveos.config_store())?;
        }
        Ok(self)
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn genesis(&self) -> &RoochGenesis {
        &self.genesis
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<ExecuteTransactionResult> {
        let tx_hash = tx.ctx.tx_hash();
        let (state_root, output) = self.moveos.execute_and_apply(tx)?;
        self.handle_tx_output(tx_hash, state_root, output)
    }

    fn handle_tx_output(
        &mut self,
        tx_hash: H256,
        state_root: H256,
        output: TransactionOutput,
    ) -> Result<ExecuteTransactionResult> {
        let event_hashes: Vec<_> = output.events.iter().map(|e| e.hash()).collect();
        let event_root = InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();

        let transaction_info = TransactionExecutionInfo::new(
            tx_hash,
            state_root,
            event_root,
            output.gas_used,
            output.status.clone(),
        );
        self.moveos
            .transaction_store()
            .save_tx_execution_info(transaction_info.clone())
            .map_err(|e| {
                anyhow::anyhow!(
                    "ExecuteTransactionMessage handler save tx info failed: {:?} {}",
                    transaction_info,
                    e
                )
            })?;
        Ok(ExecuteTransactionResult {
            output,
            transaction_info,
        })
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl Handler<ExecuteTransactionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResult> {
        self.execute(msg.tx)
    }
}
