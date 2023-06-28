// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::*;

struct TestDAService {}

impl DAClientMethods for TestDAService {
    fn put(&self, request: DAPutRequest) -> DAPutResponse {
        println!(
            "{:?} {:?} {:?}",
            request.kv, request.signature, request.checksum
        );
        let response = DAPutResponse {
            keyset_hash: vec![],
            value_hash: vec![],
            signers_mask: 0,
            signature: vec![],
            version: 0,
        };
        println!(
            "{:?} {:?} {:?} {:?} {:?}",
            response.keyset_hash,
            response.value_hash,
            response.signers_mask,
            response.signature,
            response.version
        );
        response
    }

    fn get(&self, request: DAGetRequest) -> DAGetResponse {
        println!("{:?}", request.hash);
        let response = DAGetResponse { data: vec![] };
        println!("{:?}", response.data);
        response
    }
}
