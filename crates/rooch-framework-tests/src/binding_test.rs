// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use metrics::RegistryService;
use move_core_types::account_address::AccountAddress;
use move_core_types::u256::U256;
use move_core_types::vm_status::KeptVMStatus;
use moveos_config::DataDirPath;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::{FieldKey, MoveStructType, ObjectChange, ObjectState, StateChangeSet};
use moveos_types::state_resolver::{
    RootObjectResolver, StateKV, StateReaderExt, StateResolver, StatelessResolver,
};
use moveos_types::transaction::{FunctionCall, MoveAction, VerifiedMoveOSTransaction};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::actor::{executor::ExecutorActor, messages::ExecuteTransactionResult};
use rooch_genesis::RoochGenesis;
use rooch_types::address::BitcoinAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::framework::gas_coin::RGas;
use rooch_types::framework::transfer::TransferModule;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use rooch_types::transaction::authenticator::BitcoinAuthenticator;
use rooch_types::transaction::{
    Authenticator, L1BlockWithBody, L1Transaction, RoochTransaction, RoochTransactionData,
};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::{env, vec};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tracing::info;

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
    network: RoochNetwork,
    //we keep the opt to ensure the temp dir is not be deleted before the test end
    opt: RoochOpt,
    pub sequencer: AccountAddress,
    pub sequencer_bitcoin_address: BitcoinAddress,
    kp: RoochKeyPair,
    pub executor: ExecutorActor,
    pub reader_executor: ReaderExecutorActor,
    root: ObjectMeta,
    rooch_db: RoochDB,
    pub registry_service: RegistryService,
    events: Vec<Event>,
}

impl RustBindingTest {
    // RustBindingTest new must be in a tokio runtime due to raw store dependency on tokio
    // There are two ways to ensure this:
    // 1. The upper layer calls are in the tokio runtime
    // 2. Create an independent tokio runtime when self new
    pub fn new_in_tokio() -> Result<Self> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { Self::new() })
    }

    pub fn new() -> Result<Self> {
        Self::new_with_network(BuiltinChainID::Local.into())
    }

    pub fn new_with_network(mut network: RoochNetwork) -> Result<Self> {
        let kp = RoochKeyPair::generate_secp256k1();
        let sequencer = kp.public().bitcoin_address()?;

        network.mock_genesis_account(&kp)?;

        let genesis = RoochGenesis::build(network.clone())?;
        let opt = RoochOpt::new_with_temp_store()?;
        let store_config = opt.store_config();
        let registry_service = metrics::RegistryService::default();
        let rooch_db = RoochDB::init(store_config, &registry_service.default_registry())?;
        let root = genesis.init_genesis(&rooch_db)?;

        let executor = ExecutorActor::new(
            root.clone(),
            rooch_db.moveos_store.clone(),
            rooch_db.rooch_store.clone(),
            &registry_service.default_registry(),
            None,
        )?;

        let reader_executor = ReaderExecutorActor::new(
            root.clone(),
            rooch_db.moveos_store.clone(),
            rooch_db.rooch_store.clone(),
            None,
        )?;
        Ok(Self {
            network,
            opt,
            root,
            sequencer: sequencer.to_rooch_address().into(),
            sequencer_bitcoin_address: sequencer,
            kp,
            executor,
            reader_executor,
            rooch_db,
            registry_service,
            events: vec![],
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
        RootObjectResolver::new(self.root.clone(), &self.rooch_db.moveos_store)
    }

    pub fn root(&self) -> &ObjectMeta {
        &self.root
    }

    pub fn rooch_db(&self) -> &RoochDB {
        &self.rooch_db
    }

    pub fn get_rgas(&mut self, addr: AccountAddress, amount: U256) -> Result<()> {
        // transfer RGas from rooch dao account to addr
        let function_call =
            TransferModule::create_transfer_coin_action(RGas::struct_tag(), addr, amount);
        let sender = self
            .network
            .genesis_config
            .rooch_dao
            .multisign_bitcoin_address
            .to_rooch_address();
        let sequence_number = self.get_account_sequence_number(sender.into())?;
        let tx_data = RoochTransactionData::new(
            sender,
            sequence_number,
            self.network.chain_id.id,
            GasScheduleConfig::CLI_DEFAULT_MAX_GAS_AMOUNT,
            function_call,
        );
        //RoochDao is a multisign account, so we need to sign the tx with the multisign account
        //In test env, it is a 1-of-1 multisign account, so we can sign with the only key
        let first_signature = BitcoinAuthenticator::sign(&self.kp, &tx_data);
        let authenticator = Authenticator::bitcoin_multisign(vec![first_signature])?;
        let tx = RoochTransaction::new(tx_data, authenticator);
        self.execute(tx)?;
        Ok(())
    }

    //TODO let the module bundle to execute the function
    pub fn execute(&mut self, tx: RoochTransaction) -> Result<ExecuteTransactionResult> {
        let execute_result = self.execute_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(execute_result)
    }

    pub fn execute_function_call_via_sequencer(
        &mut self,
        function_call: FunctionCall,
    ) -> Result<ExecuteTransactionResult> {
        let action = MoveAction::Function(function_call);
        let sequence_number = self.get_account_sequence_number(self.sequencer)?;
        let tx_data = RoochTransactionData::new(
            self.sequencer.into(),
            sequence_number,
            self.network.chain_id.id,
            GasScheduleConfig::CLI_DEFAULT_MAX_GAS_AMOUNT,
            action,
        );
        let tx = tx_data.sign(&self.kp);
        let result = self.execute_as_result(tx)?;
        Ok(result)
    }

    pub fn execute_l1_block_and_tx(
        &mut self,
        l1_block: L1BlockWithBody,
    ) -> Result<Vec<ExecuteTransactionResult>> {
        let l1_txs = self.execute_l1_block(l1_block.clone())?;
        let tx_len = l1_txs.len();
        let mut total_gas_used = 0;
        let mut results = vec![];
        for l1_tx in l1_txs {
            let resut = self.execute_l1_tx(l1_tx)?;
            total_gas_used += resut.output.gas_used;
            results.push(resut);
        }
        let avg_gas_used = total_gas_used / tx_len as u64;
        info!(
            "execute l1 block total gas used: {}, avg gas used: {}",
            total_gas_used, avg_gas_used
        );
        Ok(results)
    }

    pub fn execute_l1_block(&mut self, l1_block: L1BlockWithBody) -> Result<Vec<L1Transaction>> {
        let verified_tx: VerifiedMoveOSTransaction =
            self.executor.validate_l1_block(l1_block.clone())?;
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
            // Move coinbase tx to the end
            let coinbase_tx = l1_txs.pop_front().expect("coinbase tx should exist");
            l1_txs.push_back(coinbase_tx);
            Ok(l1_txs.into_iter().collect::<Vec<_>>())
        } else {
            Ok(vec![])
        }
    }

    pub fn execute_l1_tx(&mut self, l1_tx: L1Transaction) -> Result<ExecuteTransactionResult> {
        let verified_tx = self.executor.validate_l1_tx(l1_tx)?;
        self.execute_verified_tx(verified_tx)
    }

    pub fn execute_as_result(&mut self, tx: RoochTransaction) -> Result<ExecuteTransactionResult> {
        let verified_tx = self.executor.validate_l2_tx(tx)?;
        self.execute_verified_tx_as_result(verified_tx)
    }

    pub fn execute_verified_tx(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResult> {
        let execute_result = self.execute_verified_tx_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(execute_result)
    }

    pub fn execute_verified_tx_as_result(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResult> {
        let result = self.executor.execute(tx)?;
        self.root = result.transaction_info.root_metadata();
        self.events.extend(result.output.events.clone());
        self.reader_executor
            .refresh_state(self.root.clone(), false)?;
        Ok(result)
    }

    /// Directly apply a change set to the state and update root
    pub fn apply_changes(&mut self, changes: Vec<ObjectChange>) -> Result<()> {
        let mut change_set = StateChangeSet::new(self.root.state_root(), self.root.size);
        for change in changes {
            change_set.add_change(change)?;
        }
        self.rooch_db
            .moveos_store
            .state_store
            .apply_change_set(&mut change_set)?;
        self.root = change_set.root_metadata();
        self.reader_executor
            .refresh_state(self.root.clone(), false)?;
        self.executor.refresh_state(self.root.clone(), false)?;
        Ok(())
    }

    pub fn get_account_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        Ok(self
            .resolver()
            .get_account(address)?
            .map(|account| account.value.sequence_number)
            .unwrap_or(0))
    }

    pub fn events(&self) -> &Vec<Event> {
        &self.events
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

impl StateResolver for RustBindingTest {
    fn root(&self) -> &ObjectMeta {
        &self.root
    }
}

impl StatelessResolver for RustBindingTest {
    fn get_field_at(&self, state_root: H256, key: &FieldKey) -> Result<Option<ObjectState>> {
        self.resolver().get_field_at(state_root, key)
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.resolver().list_fields_at(state_root, cursor, limit)
    }
}
