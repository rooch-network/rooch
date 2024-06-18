// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::KeptVMStatus;
use moveos_config::DataDirPath;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state_resolver::{RootObjectResolver, StateReaderExt};
use moveos_types::transaction::{FunctionCall, VerifiedMoveOSTransaction};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::actor::{executor::ExecutorActor, messages::ExecuteTransactionResult};
use rooch_genesis::RoochGenesis;
use rooch_types::address::BitcoinAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use rooch_types::transaction::{L1BlockWithBody, L1Transaction, RoochTransaction};
use std::collections::VecDeque;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

pub fn get_data_dir() -> DataDirPath {
    match env::var("ROOCH_TEST_DATA_DIR") {
        Ok(path_str) => {
            let temp_dir = TempDir::new_in(path_str)
                .expect("failed to create temp dir in provided data dir path");
            DataDirPath::TempPath(Arc::from(temp_dir))
        }
        Err(_) => moveos_config::temp_dir(),
    }
}

pub struct RustBindingTest {
    //we keep the opt to ensure the temp dir is not be deleted before the test end
    opt: RoochOpt,
    sequencer: AccountAddress,
    sequencer_bitcoin_address: BitcoinAddress,
    kp: RoochKeyPair,
    pub executor: ExecutorActor,
    pub reader_executor: ReaderExecutorActor,
    root: RootObjectEntity,
    moveos_store: MoveOSStore,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let opt = RoochOpt::new_with_temp_store()?;
        let store_config = opt.store_config();
        let rooch_db = RoochDB::init(store_config)?;

        let mut network: RoochNetwork = BuiltinChainID::Local.into();

        let kp = RoochKeyPair::generate_secp256k1();
        let sequencer = kp.public().bitcoin_address()?;

        network.set_sequencer_account(sequencer.clone());

        let genesis = RoochGenesis::load_or_init(network, &rooch_db)?;
        let root = genesis.genesis_root().clone();

        let executor = ExecutorActor::new(
            root.clone(),
            rooch_db.moveos_store.clone(),
            rooch_db.rooch_store.clone(),
        )?;

        let reader_executor = ReaderExecutorActor::new(
            root.clone(),
            rooch_db.moveos_store.clone(),
            rooch_db.rooch_store,
        )?;
        Ok(Self {
            opt,
            root,
            sequencer: sequencer.to_rooch_address().into(),
            sequencer_bitcoin_address: sequencer,
            kp,
            executor,
            reader_executor,
            moveos_store: rooch_db.moveos_store,
        })
    }

    pub fn executor(&self) -> &ExecutorActor {
        &self.executor
    }

    pub fn sequencer_kp(&self) -> &RoochKeyPair {
        &self.kp
    }

    pub fn data_dir(&self) -> &Path {
        self.opt.base().data_dir()
    }

    pub fn resolver(&self) -> RootObjectResolver<MoveOSStore> {
        RootObjectResolver::new(self.root.clone(), &self.moveos_store)
    }

    pub fn root(&self) -> &RootObjectEntity {
        &self.root
    }

    //TODO let the module bundle to execute the function
    pub fn execute(&mut self, tx: RoochTransaction) -> Result<()> {
        let execute_result = self.execute_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(())
    }

    pub fn execute_l1_block(&mut self, l1_block: L1BlockWithBody) -> Result<()> {
        let ctx = self.create_l1_block_ctx(&l1_block)?;
        let verified_tx: VerifiedMoveOSTransaction = self.executor.validate_l1_block(
            ctx,
            l1_block.clone(),
            self.sequencer_bitcoin_address.clone(),
        )?;
        self.execute_verified_tx(verified_tx)?;

        if l1_block.block.chain_id.is_bitcoin() {
            let block =
                bcs::from_bytes::<rooch_types::bitcoin::types::Block>(&l1_block.block_body)?;
            let mut l1_txs = block
                .txdata
                .iter()
                .map(|tx| {
                    L1Transaction::new(
                        l1_block.block.chain_id,
                        l1_block.block.block_hash.clone(),
                        tx.id.to_vec(),
                    )
                })
                .collect::<VecDeque<_>>();
            let coinbase_tx = l1_txs.pop_front().expect("coinbase tx should exist");
            l1_txs.push_back(coinbase_tx);
            for tx in &block.txdata[1..] {
                let l1_tx = L1Transaction::new(
                    l1_block.block.chain_id,
                    l1_block.block.block_hash.clone(),
                    tx.id.to_vec(),
                );
                self.execute_l1_tx(l1_tx)?;
            }
            let coinbase_tx = L1Transaction::new(
                l1_block.block.chain_id,
                l1_block.block.block_hash.clone(),
                block.txdata[0].id.to_vec(),
            );
            self.execute_l1_tx(coinbase_tx)?;
        }
        Ok(())
    }

    pub fn execute_l1_tx(&mut self, l1_tx: L1Transaction) -> Result<()> {
        let ctx = self.create_l1_tx_ctx(l1_tx.clone())?;
        let verified_tx =
            self.executor
                .validate_l1_tx(ctx, l1_tx, self.sequencer_bitcoin_address.clone())?;
        self.execute_verified_tx(verified_tx)
    }

    pub fn create_l1_block_ctx(&self, l1_block: &L1BlockWithBody) -> Result<TxContext> {
        let sequence_number = self.get_account_sequence_number(self.sequencer)?;
        let max_gas_amount = GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT * 1000;
        let tx_hash = l1_block.block.tx_hash();
        let tx_size = l1_block.block.tx_size();

        Ok(TxContext::new(
            self.sequencer,
            sequence_number,
            max_gas_amount,
            tx_hash,
            tx_size,
        ))
    }

    pub fn create_l1_tx_ctx(&self, l1_tx: L1Transaction) -> Result<TxContext> {
        let sequence_number = self.get_account_sequence_number(self.sequencer)?;
        let max_gas_amount = GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT * 1000;
        let tx_hash = l1_tx.tx_hash();
        let tx_size = l1_tx.tx_size();

        Ok(TxContext::new(
            self.sequencer,
            sequence_number,
            max_gas_amount,
            tx_hash,
            tx_size,
        ))
    }

    pub fn execute_as_result(&mut self, tx: RoochTransaction) -> Result<ExecuteTransactionResult> {
        let verified_tx = self.executor.validate_l2_tx(tx)?;
        self.execute_verified_tx_as_result(verified_tx)
    }

    pub fn execute_verified_tx(&mut self, tx: VerifiedMoveOSTransaction) -> Result<()> {
        let execute_result = self.execute_verified_tx_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(())
    }

    pub fn execute_verified_tx_as_result(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResult> {
        let result = self.executor.execute(tx)?;
        let root = ObjectEntity::root_object(
            result.transaction_info.state_root,
            result.transaction_info.size,
        );
        self.reader_executor.refresh_state(root.clone(), false)?;
        self.root = root;
        Ok(result)
    }

    pub fn get_account_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        Ok(self
            .resolver()
            .get_account(address)?
            .map(|account| account.value.sequence_number)
            .unwrap_or(0))
    }
}

impl MoveFunctionCaller for RustBindingTest {
    fn call_function(
        &self,
        ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let result = self.reader_executor.moveos().execute_readonly_function(
            self.root.clone(),
            ctx,
            function_call,
        );
        Ok(result)
    }
}
