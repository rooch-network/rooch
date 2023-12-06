use moveos_types::h256::H256;

// Request to put batch to the DA server
pub struct BatchPutRequest {
    // The version of this struct.
    pub version: u8,
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    pub batch: Vec<u8>,

    // TODO add put policy

    // signature result of BatchPutRequest
    pub signature: Vec<u8>,
}

// Response to put batch to the DA server
pub struct BatchPutResponse {
    // The version of this struct.
    pub version: u8,
    // TODO checksum algorithm
    // checksum of the batch data: help to check publication integrity
    pub checksum: Vec<u8>,

    // signature result of BatchPutResponse
    pub signature: Vec<u8>,

}

// TODO get request and response
// 1. get by block number
// 2. get by batch hash
// 3. pull by stream
//
// TODO ECC for SDC protection (wrong response attacks)

// client interface that make requests of batch to DA server
// TODO async trait
pub trait BatchClient {
    fn put(&self, request: BatchPutRequest) -> BatchPutResponse;
}