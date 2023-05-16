struct SendTxnRequest {
    sender_address: String,
    public_key: Vec<u8>,
    program: Vec<u8>,
    sequencer_number: u64,
    expiration_time: u64,
    signature: Vec<u8>,
}
struct SendTxnResponse {
    // The following four fields together are completed legal txn order evidence.
    // Transaction Hash
    txn_hash: String,
    // user's transaction's order: the execution order will follow this order strictly.
    // e.g. txn with y order must be no later than txn with x order, if x > y
    order: u128,
    // It hash of txn_hash and order for indexing txn and order pair
    // It's SHA256(txn_hash, order), txn_hash & order are inputs of hash function.
    order_hash: String,
    // Order signed by sequencer.
    txn_order_signature: Vec<u8>,
}

// GetTxnOrderWitnessRequest requests sequencer for txn's order witness for ensuring txn's order is in the right place,
// anyone could verify it.
// Witness is Layer1-friendly which could verify it in a fast way and slashing dishonest sequencer.
// The Content of this struct is as same as SendTxnResponse
struct GetTxnOrderWitnessRequest {
    txn_hash: String,
    order: u128,
    order_hash: String,
    txn_order_signature: Vec<u8>,
}

struct GetTxnOrderWitnessResponse {
    txn_hash: String,
    order: u128,
    order_hash: String,
    // Based on the order information of the transactions provided by the user,
    // return a verifiable proof of transaction order,
    // so the user can verify whether the order is valid or not.
    // And the result could be "nil" means sequencer cannot give the witness, may caused by:
    // illegal TxnOrder
    // internal server error
    // too old txn need to search DA, but bad connection or heavy traffic cause request failed
    txn_order_witness: Option<Vec<u8>>,
    error_message: Option<String>,
    // Order signed by sequencer.
    txn_order_signature: Vec<u8>,
}

struct TxnInfoRequest {
    txn_hash: String,
}

struct TxnInfoResponse {
    block_hash: String,
    block_number: u64,
    txn_hash: String,
    user_txn: Vec<u8>,
    conformations: u64,
}

struct BlockByNumberRequest {
    block_number: u64,
}
struct BlockResponse {
    block_hash: String,
    parent_hash: String,
    timestamp: u64,
    number: u64,
    txn_accumulator_root: String,
    block_accumulator_root: String,
    state_root: String,
    gas_used: u64,
    nonce: u64,
    chain_id: u64,
    confirmations: u64,
}

struct BlockByHashRequest {
    hash: u64,
}

// Query account balance at specified block height
struct AccountBalanceRequest {
    account: String,
    block: Option<u64>,
}

struct AccountBalanceResponse {
    balance: String,
}

struct AccountSequenceNumberRequest {
    account: String,
}

struct AccountSequenceNumberResponse {
    sequence_number: u128,
}

struct Txn {
    txn_hash: String,
    user_txn: Vec<u8>,
}

// Execute user transaction at specified block height
struct ContractCallRequest {
    txn: Txn,
    block: Option<u128>,
}

struct ContractCallResponse {
    txn_hash: String,
    user_txn: Vec<u8>,
}

// query account proof at specified block height
struct AccountProofRequest {
    // Account address
    account: String,
    // Array of storage-keys which should be proofed and included.
    storage_keys: Vec<Vec<u8>>,
    // Optioned block number
    block: Option<u128>,
}

struct Proof {
    // The requested storage key
    key: String,
    // The storage value
    value: String,
    // Array of rlp-serialized MerkleTree-Nodes, starting with the storageHash-Node,
    // following the path of the SHA3 (key) as path.
    proof: Vec<Vec<u8>>,
}

struct AccountProofResponse {
    // Account address
    account: String,
    // Array of rlp-serialized MerkleTree-Nodes, starting with the stateRoot-Node,
    // following the path of the SHA3 (address) as key.
    account_proof: Vec<Vec<u8>>,
    // Account balance
    balance: u128,
    // Sequence number of account
    sequence_number: u128,
    // SHA3 of the StorageRoot.
    // All storage will deliver a MerkleProof starting with this rootHash.
    storage_hash: Vec<u8>,
    // Array of storage-entries as requested.
    storage_proof: Vec<Proof>,
}

trait MoveOSClient {
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

struct RPCRequest<T> {
    id: u32,
    json_rpc: String,
    method: String,
    params: Vec<T>,
}

impl<T> RPCRequest<T> {
    fn new_request(method: String, params: Vec<T>) -> Self {
        Self {
            id: 0,
            json_rpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

struct RPCResponse<T> {
    id: u32,
    json_rpc: String,
    result: Option<T>,
}

trait RPCClient<T> {
    fn send_request(&self, request: RPCRequest<T>, timeout: u64) -> RPCResponse<T>;
}
