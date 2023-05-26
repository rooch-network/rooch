// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos::moveos::TransactionOutput;
use moveos_types::object::ObjectID;
use rooch_executor::proxy::ExecutorProxy;
use rooch_proposer::proxy::ProposerProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::{address::RoochAddress, transaction::TypedTransaction};

/// RpcService is the implementation of the RPC service.
/// It is the glue between the RPC server(EthAPIServer,RoochApiServer) and the rooch's actors.
/// The RpcService encapsulates the logic of the functions, and the RPC server handle the response format.
#[derive(Clone)]
pub struct RpcService {
    executor: ExecutorProxy,
    sequencer: SequencerProxy,
    proposer: ProposerProxy,
}

impl RpcService {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        proposer: ProposerProxy,
    ) -> Self {
        Self {
            executor,
            sequencer,
            proposer,
        }
    }
}

impl RpcService {
    pub async fn quene_tx(&self, tx: TypedTransaction) -> Result<()> {
        //TODO implement quene tx and do not wait to execute
        let _ = self.execute_tx(tx).await?;
        Ok(())
    }

    pub async fn execute_tx(&self, tx: TypedTransaction) -> Result<TransactionOutput> {
        //First, validate the transactin
        let moveos_tx = self.executor.validate_transaction(tx.clone()).await?;
        let tx_sequence_info = self.sequencer.sequence_transaction(tx.clone()).await?;
        // Then execute
        let (output, tx_execution_info) = self.executor.execute_transaction(moveos_tx).await?;
        self.proposer
            .propose_transaction(tx, tx_execution_info, tx_sequence_info)
            .await?;
        //TODO conform the response, put the sequence result to output.
        Ok(output)
    }

    pub async fn view(&self, payload: Vec<u8>) -> Result<Vec<serde_json::Value>> {
        let output_values = self.executor.view(payload).await?;
        let mut resp = vec![];
        for v in output_values {
            resp.push(serde_json::to_value(v)?);
        }
        Ok(resp)
    }

    pub async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> Result<String> {
        let resp = self
            .executor
            .resource(address, &module, &resource, type_args)
            .await?;
        Ok(resp)
    }

    pub async fn object(&self, object_id: ObjectID) -> Result<String> {
        let resp = self.executor.object(object_id).await?;
        //TODO return Move Object
        Ok(resp)
    }

    /// Sign a message with the private key of the given address.
    pub async fn sign(&self, _address: RoochAddress, _message: Vec<u8>) -> Result<Vec<u8>> {
        bail!("Not implemented")
        //TODO implement sign
        //Call WalletActor to sign?
        //How to unlock the wallet?
        //Define the sign message format for rooch, and does it need to be compatible with Ethereum?
    }

    pub async fn accounts(&self) -> Result<Vec<RoochAddress>> {
        bail!("Not implemented")
    }
}
