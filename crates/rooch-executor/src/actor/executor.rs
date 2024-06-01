// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    ExecuteTransactionMessage, ExecuteTransactionResult, GetRootMessage, ValidateL1BlockMessage,
    ValidateL2TxMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::vm_status::VMStatus;
use moveos::moveos::{MoveOS, MoveOSConfig};
use moveos::vm::vm_status_explainer::explain_vm_status;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::transaction::{FunctionCall, MoveOSTransaction, VerifiedMoveAction};
use rooch_genesis::FrameworksGasParameters;
use rooch_store::RoochStore;
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::BitcoinModule;
use rooch_types::framework::auth_validator::{AuthValidatorCaller, TxValidateResult};
use rooch_types::framework::ethereum::EthereumModule;
use rooch_types::framework::transaction_validator::TransactionValidator;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::{AuthenticatorInfo, L1Block, L1BlockWithBody, RoochTransaction};
use tracing::{debug, warn};

pub struct ExecutorActor {
    root: RootObjectEntity,
    moveos: MoveOS,
    moveos_store: MoveOSStore,
    rooch_store: RoochStore,
}

type ValidateAuthenticatorResult =
    Result<(TxValidateResult, Vec<FunctionCall>, Vec<FunctionCall>), VMStatus>;

impl ExecutorActor {
    pub fn new(
        root: RootObjectEntity,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let resolver = RootObjectResolver::new(root.clone(), &moveos_store);
        let gas_parameters = FrameworksGasParameters::load_from_chain(&resolver)?;

        let moveos = MoveOS::new(
            moveos_store.clone(),
            gas_parameters.all_natives(),
            MoveOSConfig::default(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
        )?;

        Ok(Self {
            root,
            moveos,
            moveos_store,
            rooch_store,
        })
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn get_moveos_store(&self) -> MoveOSStore {
        self.moveos.moveos_store().clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<ExecuteTransactionResult> {
        let tx_hash = tx.ctx.tx_hash();
        let (state_root, size, output) = self.moveos.execute_and_apply(tx)?;
        let execution_info =
            self.moveos_store
                .handle_tx_output(tx_hash, state_root, size, output.clone())?;

        self.root = execution_info.root_object();
        Ok(ExecuteTransactionResult {
            output,
            transaction_info: execution_info,
        })
    }

    pub fn validate_l1_block(
        &self,
        mut ctx: TxContext,
        l1_block: L1BlockWithBody,
        sequencer_address: BitcoinAddress,
    ) -> Result<VerifiedMoveOSTransaction> {
        ctx.add(TxValidateResult::new_l1_block(sequencer_address))?;
        //In the future, we should verify the block PoW difficulty or PoS validator signature before the sequencer decentralized
        let L1BlockWithBody {
            block:
                L1Block {
                    chain_id,
                    block_height,
                    block_hash,
                },
            block_body,
        } = l1_block;
        match RoochMultiChainID::try_from(chain_id.id())? {
            RoochMultiChainID::Bitcoin => {
                let action = VerifiedMoveAction::Function {
                    call: BitcoinModule::create_submit_new_block_call_bytes(
                        block_height,
                        block_hash,
                        block_body,
                    )?,
                    bypass_visibility: true,
                };
                Ok(VerifiedMoveOSTransaction::new(
                    self.root.clone(),
                    ctx,
                    action,
                ))
            }
            RoochMultiChainID::Ether => {
                let action = VerifiedMoveAction::Function {
                    call: EthereumModule::create_submit_new_block_call_bytes(block_body),
                    bypass_visibility: true,
                };
                Ok(VerifiedMoveOSTransaction::new(
                    self.root.clone(),
                    ctx,
                    action,
                ))
            }
            id => Err(anyhow::anyhow!("Chain {} not supported yet", id)),
        }
    }

    pub fn validate_l2_tx(&self, mut tx: RoochTransaction) -> Result<VerifiedMoveOSTransaction> {
        let sender = tx.sender();
        let tx_hash = tx.tx_hash();

        debug!("executor validate_l2_tx: {:?}, sender: {}", tx_hash, sender);

        let authenticator = tx.authenticator_info();

        let mut moveos_tx: MoveOSTransaction = tx.into_moveos_transaction(self.root.clone());
        let result = self.validate_authenticator(&moveos_tx.ctx, authenticator);
        match result {
            Ok(vm_result) => match vm_result {
                Ok((tx_validate_result, pre_execute_functions, post_execute_functions)) => {
                    // Add the tx_validate_result to the context
                    moveos_tx
                        .ctx
                        .add(tx_validate_result)
                        .expect("add tx_validate_result failed");

                    moveos_tx.append_pre_execute_functions(pre_execute_functions);
                    moveos_tx.append_post_execute_functions(post_execute_functions);
                    let verify_result = self.moveos.verify(moveos_tx);
                    match verify_result {
                        Ok(verified_tx) => Ok(verified_tx),
                        Err(e) => {
                            log::warn!(
                                "transaction verify vm error, tx_hash: {}, error:{:?}",
                                tx_hash,
                                e
                            );
                            Err(e.into())
                        }
                    }
                }
                Err(e) => {
                    let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
                    let status_view = explain_vm_status(&resolver, e.clone())?;
                    warn!(
                        "transaction validate vm error, tx_hash: {}, error:{:?}",
                        tx_hash, status_view,
                    );
                    //TODO how to return the vm status to rpc client.
                    Err(e.into())
                }
            },
            Err(e) => {
                log::warn!(
                    "transaction validate error, tx_hash: {}, error:{:?}",
                    tx_hash,
                    e
                );
                Err(e)
            }
        }
    }

    pub fn validate_authenticator(
        &self,
        ctx: &TxContext,
        authenticator: AuthenticatorInfo,
    ) -> Result<ValidateAuthenticatorResult> {
        let tx_validator = self.as_module_binding::<TransactionValidator>();
        let tx_validate_function_result = tx_validator
            .validate(ctx, authenticator.clone())?
            .into_result();

        let vm_result = match tx_validate_function_result {
            Ok(tx_validate_result) => {
                let auth_validator_option = tx_validate_result.auth_validator();
                match auth_validator_option {
                    Some(auth_validator) => {
                        let auth_validator_caller = AuthValidatorCaller::new(self, auth_validator);
                        let auth_validator_function_result = auth_validator_caller
                            .validate(ctx, authenticator.authenticator.payload)?
                            .into_result();
                        match auth_validator_function_result {
                            Ok(_) => {
                                // pre_execute_function: AuthValidator
                                let pre_execute_functions =
                                    vec![auth_validator_caller.pre_execute_function_call()];
                                // post_execute_function: AuthValidator
                                let post_execute_functions =
                                    vec![auth_validator_caller.post_execute_function_call()];
                                Ok((
                                    tx_validate_result,
                                    pre_execute_functions,
                                    post_execute_functions,
                                ))
                            }
                            Err(vm_status) => Err(vm_status),
                        }
                    }
                    None => {
                        let pre_execute_functions = vec![];
                        let post_execute_functions = vec![];
                        Ok((
                            tx_validate_result,
                            pre_execute_functions,
                            post_execute_functions,
                        ))
                    }
                }
            }
            Err(vm_status) => Err(vm_status),
        };
        Ok(vm_result)
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl Handler<ValidateL2TxMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ValidateL2TxMessage,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate_l2_tx(msg.tx)
    }
}

#[async_trait]
impl Handler<ValidateL1BlockMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ValidateL1BlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate_l1_block(msg.ctx, msg.l1_block, msg.sequencer_address)
    }
}

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

#[async_trait]
impl Handler<GetRootMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        _msg: GetRootMessage,
        _ctx: &mut ActorContext,
    ) -> Result<RootObjectEntity> {
        Ok(self.root.clone())
    }
}

impl MoveFunctionCaller for ExecutorActor {
    fn call_function(&self, ctx: &TxContext, call: FunctionCall) -> Result<FunctionResult> {
        Ok(self
            .moveos
            .execute_readonly_function(self.root.clone(), ctx, call))
    }
}
