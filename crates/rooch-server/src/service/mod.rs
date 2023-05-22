// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::RoochRpcModule;
use crate::response::JsonResponse;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::RpcModule;
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
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::TypedTransaction;

// Define a rpc server api
#[rpc(server, client)]
pub trait RpcService {
    #[method(name = "echo")]
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>>;

    #[method(name = "submit_txn")]
    async fn submit_txn(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<TransactionOutput>>;

    #[method(name = "view")]
    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<Vec<serde_json::Value>>>;

    #[method(name = "resource")]
    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<JsonResponse<String>>;

    #[method(name = "object")]
    async fn object(&self, object_id: ObjectID) -> RpcResult<JsonResponse<String>>;
}

pub struct RoochServer {
    executor: ExecutorProxy,
    sequencer: SequencerProxy,
    proposer: ProposerProxy,
}

impl RoochServer {
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

#[async_trait]
impl RpcServiceServer for RoochServer {
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>> {
        Ok(JsonResponse::ok(msg))
    }

    async fn submit_txn(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<TransactionOutput>> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload).map_err(anyhow::Error::from)?;
        println!("sender: {:?}", tx.sender());
        //First, validate the transactin
        let moveos_tx = self.executor.validate_transaction(tx.clone()).await?;
        let typed_tx = TypedTransaction::Rooch(tx);
        let tx_sequence_info = self
            .sequencer
            .sequence_transaction(typed_tx.clone())
            .await?;
        // Then execute
        let (output, tx_execution_info) = self.executor.execute_transaction(moveos_tx).await?;
        self.proposer
            .propose_transaction(typed_tx, tx_execution_info, tx_sequence_info)
            .await?;
        //TODO conform the response, put the sequence result to output.
        Ok(JsonResponse::ok(output))
    }

    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<Vec<serde_json::Value>>> {
        let output_values = self.executor.view(payload).await?;
        let mut resp = vec![];
        for v in output_values {
            resp.push(serde_json::to_value(v)?);
        }
        // println!("{}", resp);
        Ok(JsonResponse::ok(resp))
    }

    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<JsonResponse<String>> {
        let resp = self
            .executor
            .resource(address, &module, &resource, type_args)
            .await?;
        Ok(JsonResponse::ok(resp))
    }

    async fn object(&self, object_id: ObjectID) -> RpcResult<JsonResponse<String>> {
        let resp = self.executor.object(object_id).await?;
        Ok(JsonResponse::ok(resp))
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
