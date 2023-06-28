// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod test;

// Request to store data to the DA server
pub struct DAPutRequest {
    // Aggregation of key and value contents.
    pub kv: Vec<u8>,
    // Used to check the integrity of data in the application layer code.
    pub checksum: u64,
    // The signature result of DAPutRequest for DA client.
    pub signature: Vec<u8>,
}

// Data storage response returned by the DA server
pub struct DAPutResponse {
    // The hash of multi-signature collection returned by multiple nodes comprising the DA committee.
    // The actual public key collection is stored in DA and retrieved from it as needed for verification.
    pub keyset_hash: Vec<u8>,
    // Hash of the data returned by the DA Server.
    pub value_hash: Vec<u8>,
    // Use signersMask to extract the corresponding public key set from the keyset data.
    pub signers_mask: u64,
    // Multi-signature of the DA committee.
    pub signature: Vec<u8>,
    // The version of this struct.
    pub version: u8,
}

pub struct DAGetRequest {
    pub hash: Vec<u8>,
}

pub struct DAGetResponse {
    pub data: Vec<u8>,
}

pub trait DAClientMethods {
    fn put(&self, request: DAPutRequest) -> DAPutResponse;
    fn get(&self, request: DAGetRequest) -> DAGetResponse;
}
