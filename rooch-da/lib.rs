// Request to store data to the DA server
struct DAStoreRequest {
    message: Vec<u8>,
    timeout: u64,
    signature: Vec<u8>,
}

// Data storage response returned by the DA server
struct DAStoreResponse {
    // The hash of multi-signature collection returned by multiple nodes comprising the DA committee.
    // The actual public key collection is stored in DA and retrieved from it as needed for verification.
    keyset_hash: Vec<u8>,
    // Hash of the data returned by the DA Server.
    data_hash: Vec<u8>,
    // The timeout period for the storage backend.
    timeout: u64,
    // Use signersMask to extract the corresponding public key set from the keyset data.
    signers_mask: u64,
    // Multi-signature of the DA committee.
    sig: Vec<u8>,
    // The version of this struct.
    version: u8,
}

struct DAGetRequest {
    hash: Vec<u8>,
}

struct DAGetResponse {
    data: Vec<u8>,
}

struct DAStatusResponse {
    status_code: u64,
    status_data: Vec<u8>,
}

trait DAClientMethods {
    fn store(&self, request: DAStoreRequest) -> DAStoreResponse;
    fn get(&self, request: DAGetRequest) -> DAGetResponse;
    fn status(&self) -> DAStatusResponse;
}
