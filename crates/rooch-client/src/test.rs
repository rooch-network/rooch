use crate::client::*;

struct TestClient {}

impl MoveOSClient for TestClient {
    fn send_txn(&self, request: SendTxnRequest) -> SendTxnResponse {
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            request.program,
            request.sequencer_number,
            request.signature,
            request.public_key,
            request.sender_address,
            request.expiration_time
        );
        let response = SendTxnResponse {
            txn_hash: "".to_string(),
            order: 0,
            order_hash: "".to_string(),
            txn_order_signature: vec![],
        };
        println!(
            "{:?} {:?} {:?} {:?}",
            response.order, response.order_hash, response.txn_hash, response.txn_order_signature
        );
        response
    }

    fn get_txn_order_witness(
        &self,
        request: GetTxnOrderWitnessRequest,
    ) -> GetTxnOrderWitnessResponse {
        println!(
            "{:?} {:?} {:?} {:?}",
            request.txn_hash, request.order, request.order_hash, request.txn_order_signature,
        );
        let response = GetTxnOrderWitnessResponse {
            txn_hash: "".to_string(),
            order: 0,
            order_hash: "".to_string(),
            txn_order_witness: None,
            error_message: None,
            txn_order_signature: vec![],
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            response.txn_hash,
            response.order,
            response.order_hash,
            response.txn_order_witness,
            response.error_message,
            response.txn_order_signature
        );
        response
    }

    fn txn_info(&self, request: TxnInfoRequest) -> TxnInfoResponse {
        println!("{:?}", request.txn_hash);
        let response = TxnInfoResponse {
            block_hash: "".to_string(),
            block_number: 0,
            txn_hash: "".to_string(),
            user_txn: vec![],
            conformations: 0,
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?}",
            response.txn_hash,
            response.block_number,
            response.txn_hash,
            response.user_txn,
            response.conformations,
        );
        response
    }

    fn block_by_number(&self, request: BlockByNumberRequest) -> BlockResponse {
        println!("{:?}", request.block_number);
        let response = BlockResponse {
            block_hash: "".to_string(),
            parent_hash: "".to_string(),
            timestamp: 0,
            number: 0,
            txn_accumulator_root: "".to_string(),
            block_accumulator_root: "".to_string(),
            state_root: "".to_string(),
            gas_used: 0,
            nonce: 0,
            chain_id: 0,
            confirmations: 0,
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            response.block_hash,
            response.parent_hash,
            response.timestamp,
            response.number,
            response.txn_accumulator_root,
            response.block_accumulator_root,
            response.state_root,
            response.gas_used,
            response.nonce,
            response.chain_id,
            response.confirmations,
        );
        response
    }

    fn block_by_hash(&self, request: BlockByHashRequest) -> BlockResponse {
        println!("{:?}", request.hash);
        let response = BlockResponse {
            block_hash: "".to_string(),
            parent_hash: "".to_string(),
            timestamp: 0,
            number: 0,
            txn_accumulator_root: "".to_string(),
            block_accumulator_root: "".to_string(),
            state_root: "".to_string(),
            gas_used: 0,
            nonce: 0,
            chain_id: 0,
            confirmations: 0,
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            response.block_hash,
            response.parent_hash,
            response.timestamp,
            response.number,
            response.txn_accumulator_root,
            response.block_accumulator_root,
            response.state_root,
            response.gas_used,
            response.nonce,
            response.chain_id,
            response.confirmations,
        );
        response
    }

    fn account_balance(&self, request: AccountBalanceRequest) -> AccountBalanceResponse {
        println!("{:?} {:?}", request.account, request.block);
        let response = AccountBalanceResponse {
            balance: "".to_string(),
        };
        println!("{:?}", response.balance);
        response
    }

    fn account_sequence_number(
        &self,
        request: AccountSequenceNumberRequest,
    ) -> AccountSequenceNumberResponse {
        println!("{:?}", request.account);
        let response = AccountSequenceNumberResponse { sequence_number: 0 };
        println!("{:?}", response.sequence_number);
        response
    }

    fn call_contract(&self, request: ContractCallRequest) -> ContractCallResponse {
        println!(
            "{:?} {:?} {:?}",
            request.txn.user_txn, request.txn.txn_hash, request.block
        );
        let response = ContractCallResponse {
            txn_hash: "".to_string(),
            user_txn: vec![],
        };
        println!("{:?} {:?}", response.txn_hash, response.user_txn);
        response
    }

    fn account_proof(&self, request: AccountProofRequest) -> AccountProofResponse {
        println!(
            "{:?} {:?} {:?}",
            request.account, request.block, request.storage_keys
        );
        let response = AccountProofResponse {
            account: "".to_string(),
            account_proof: vec![],
            balance: 0,
            sequence_number: 0,
            storage_hash: vec![],
            storage_proof: vec![],
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            response.account,
            response.account_proof,
            response.balance,
            response.sequence_number,
            response.storage_hash,
            response.storage_proof,
        );
        response
    }
}
