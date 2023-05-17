#[derive(Clone, Debug)]
pub struct SendTxnRequest {
    pub sender_address: String,
    pub public_key: Vec<u8>,
    pub program: Vec<u8>,
    pub sequencer_number: u64,
    pub expiration_time: u64,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SendTxnResponse {
    // The following four fields together are completed legal txn order evidence.
    // Transaction Hash
    pub txn_hash: String,
    // user's transaction's order: the execution order will follow this order strictly.
    // e.g. txn with y order must be no later than txn with x order, if x > y
    pub order: u128,
    // It hash of txn_hash and order for indexing txn and order pair
    // It's SHA256(txn_hash, order), txn_hash & order are inputs of hash function.
    pub order_hash: String,
    // Order signed by sequencer.
    pub txn_order_signature: Vec<u8>,
}

// GetTxnOrderWitnessRequest requests sequencer for txn's order witness for ensuring txn's order is in the right place,
// anyone could verify it.
// Witness is Layer1-friendly which could verify it in a fast way and slashing dishonest sequencer.
// The Content of this struct is as same as SendTxnResponse
#[derive(Clone, Debug)]
pub struct GetTxnOrderWitnessRequest {
    pub txn_hash: String,
    pub order: u128,
    pub order_hash: String,
    pub txn_order_signature: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct GetTxnOrderWitnessResponse {
    pub txn_hash: String,
    pub order: u128,
    pub order_hash: String,
    // Based on the order information of the transactions provided by the user,
    // return a verifiable proof of transaction order,
    // so the user can verify whether the order is valid or not.
    // And the result could be "nil" means sequencer cannot give the witness, may caused by:
    // illegal TxnOrder
    // internal server error
    // too old txn need to search DA, but bad connection or heavy traffic cause request failed
    pub txn_order_witness: Option<Vec<u8>>,
    pub error_message: Option<String>,
    // Order signed by sequencer.
    pub txn_order_signature: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct TxnInfoRequest {
    pub txn_hash: String,
}

#[derive(Clone, Debug)]
pub struct TxnInfoResponse {
    pub block_hash: String,
    pub block_number: u64,
    pub txn_hash: String,
    pub user_txn: Vec<u8>,
    pub conformations: u64,
}

#[derive(Clone, Debug)]
pub struct BlockByNumberRequest {
    pub block_number: u64,
}

#[derive(Clone, Debug)]
pub struct BlockResponse {
    pub block_hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub number: u64,
    pub txn_accumulator_root: String,
    pub block_accumulator_root: String,
    pub state_root: String,
    pub gas_used: u64,
    pub nonce: u64,
    pub chain_id: u64,
    pub confirmations: u64,
}

#[derive(Clone, Debug)]
pub struct BlockByHashRequest {
    pub hash: u64,
}

// Query account balance at specified block height
#[derive(Clone, Debug)]
pub struct AccountBalanceRequest {
    pub account: String,
    pub block: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct AccountBalanceResponse {
    pub balance: String,
}

#[derive(Clone, Debug)]
pub struct AccountSequenceNumberRequest {
    pub account: String,
}

#[derive(Clone, Debug)]
pub struct AccountSequenceNumberResponse {
    pub sequence_number: u128,
}

#[derive(Clone, Debug)]
pub struct Txn {
    pub txn_hash: String,
    pub user_txn: Vec<u8>,
}

// Execute user transaction at specified block height
#[derive(Clone, Debug)]
pub struct ContractCallRequest {
    pub txn: Txn,
    pub block: Option<u128>,
}

#[derive(Clone, Debug)]
pub struct ContractCallResponse {
    pub txn_hash: String,
    pub user_txn: Vec<u8>,
}

// query account proof at specified block height
#[derive(Clone, Debug)]
pub struct AccountProofRequest {
    // Account address
    pub account: String,
    // Array of storage-keys which should be proofed and included.
    pub storage_keys: Vec<Vec<u8>>,
    // Optioned block number
    pub block: Option<u128>,
}

#[derive(Clone, Debug)]
pub struct Proof {
    // The requested storage key
    pub key: String,
    // The storage value
    pub value: String,
    // Array of rlp-serialized MerkleTree-Nodes, starting with the storageHash-Node,
    // following the path of the SHA3 (key) as path.
    pub proof: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct AccountProofResponse {
    // Account address
    pub account: String,
    // Array of rlp-serialized MerkleTree-Nodes, starting with the stateRoot-Node,
    // following the path of the SHA3 (address) as key.
    pub account_proof: Vec<Vec<u8>>,
    // Account balance
    pub balance: u128,
    // Sequence number of account
    pub sequence_number: u128,
    // SHA3 of the StorageRoot.
    // All storage will deliver a MerkleProof starting with this rootHash.
    pub storage_hash: Vec<u8>,
    // Array of storage-entries as requested.
    pub storage_proof: Vec<Proof>,
}

pub trait MoveOSClient {
    fn send_txn(&self, request: SendTxnRequest) -> SendTxnResponse;
    fn get_txn_order_witness(
        &self,
        request: GetTxnOrderWitnessRequest,
    ) -> GetTxnOrderWitnessResponse;
    fn txn_info(&self, request: TxnInfoRequest) -> TxnInfoResponse;
    fn block_by_number(&self, request: BlockByNumberRequest) -> BlockResponse;
    fn block_by_hash(&self, request: BlockByHashRequest) -> BlockResponse;
    fn account_balance(&self, request: AccountBalanceRequest) -> AccountBalanceResponse;
    fn account_sequence_number(
        &self,
        request: AccountSequenceNumberRequest,
    ) -> AccountSequenceNumberResponse;
    fn call_contract(&self, request: ContractCallRequest) -> ContractCallResponse;
    fn account_proof(&self, request: AccountProofRequest) -> AccountProofResponse;
}
