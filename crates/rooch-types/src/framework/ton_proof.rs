// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use base64::prelude::*;
use fastcrypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::{HashFunction, Sha256},
    traits::{ToFromBytes, VerifyingKey},
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use tonlib_core::{
    cell::BagOfCells,
    wallet::{WalletDataHighloadV2R2, WalletDataV1V2, WalletDataV3, WalletDataV4, WalletVersion},
    TonAddress,
};

const PROOF_TTL: u64 = 3600; // 1 hour

static KNOWN_HASHES: Lazy<HashMap<[u8; 32], WalletVersion>> = Lazy::new(|| {
    let mut known_hashes = HashMap::new();
    let all_versions = [
        WalletVersion::V1R1,
        WalletVersion::V1R2,
        WalletVersion::V1R3,
        WalletVersion::V2R1,
        WalletVersion::V2R2,
        WalletVersion::V3R1,
        WalletVersion::V3R2,
        WalletVersion::V4R1,
        WalletVersion::V4R2,
        WalletVersion::HighloadV1R1,
        WalletVersion::HighloadV1R2,
        WalletVersion::HighloadV2,
        WalletVersion::HighloadV2R1,
        WalletVersion::HighloadV2R2,
    ];
    all_versions.into_iter().for_each(|v| {
        let hash: [u8; 32] = v.code().unwrap().cell_hash();
        known_hashes.insert(hash, v);
    });
    known_hashes
});

#[derive(Deserialize)]
pub struct TonProofData {
    pub name: String,
    pub proof: TonProof,
    pub state_init: String,
}

const PAYLOAD_MESSAGE_IDX: u64 = 0;
const PAYLOAD_BITCOIN_ADDRESS_IDX: u64 = 1;
const PAYLOAD_TX_HASH_IDX: u64 = 2;

#[derive(Deserialize)]
pub struct TonProof {
    pub timestamp: u64,
    pub domain: TonDomain,
    pub signature: String,
    pub payload: Vec<String>,
}

#[derive(Deserialize)]
pub struct TonDomain {
    pub length_bytes: u64,
    pub value: String,
}

pub fn verify_proof(address: TonAddress, data: TonProofData, verify_timestamp: bool) -> Result<()> {
    let proof = data.proof;
    let state_init = data.state_init;

    // check ton proof expiration
    if verify_timestamp {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if now > proof.timestamp + PROOF_TTL {
            bail!("ton proof has been expired");
        }
    }

    if proof.domain.length_bytes != proof.domain.value.len() as u64 {
        bail!(
            "domain length {} mismatched against provided length bytes of {}",
            proof.domain.value.len(),
            proof.domain.length_bytes
        );
    }

    let ton_proof_prefix = "ton-proof-item-v2/";
    let mut msg: Vec<u8> = Vec::new();
    msg.extend_from_slice(ton_proof_prefix.as_bytes());
    msg.extend_from_slice(&address.workchain.to_be_bytes());
    msg.extend_from_slice(&address.hash_part);
    msg.extend_from_slice(&(proof.domain.length_bytes as u32).to_le_bytes());
    msg.extend_from_slice(proof.domain.value.as_bytes());
    msg.extend_from_slice(&proof.timestamp.to_le_bytes());
    msg.extend_from_slice(proof.payload.as_bytes());

    let mut hasher = Sha256::new();
    hasher.update(msg);
    let msg_hash = hasher.finalize();

    let mut full_msg: Vec<u8> = vec![0xff, 0xff];
    let ton_connect_prefix = "ton-connect";
    full_msg.extend_from_slice(ton_connect_prefix.as_bytes());
    full_msg.extend_from_slice(msg_hash.as_ref());

    let mut hasher = Sha256::new();
    hasher.update(full_msg);
    let full_msg_hash = hasher.finalize();

    let pubkey_bytes = {
        let bytes = BASE64_STANDARD.decode(&state_init)?;
        let boc = BagOfCells::parse(&bytes)?;
        let hash: [u8; 32] = boc.single_root()?.cell_hash();

        if hash != address.hash_part {
            return Err(anyhow!(
                "wrong address in state_init: {}",
                hex::encode(hash)
            ));
        }

        let root = boc.single_root().expect("checked above");
        let code = root.reference(0)?;
        let data = root.reference(1)?.as_ref().clone();

        let code_hash: [u8; 32] = code.cell_hash();
        let known_hashes = &*KNOWN_HASHES;
        let version = known_hashes
            .get(&code_hash)
            .ok_or(anyhow!("not known wallet version"))?
            .clone();

        match version {
            WalletVersion::V1R1
            | WalletVersion::V1R2
            | WalletVersion::V1R3
            | WalletVersion::V2R1
            | WalletVersion::V2R2 => {
                let data = WalletDataV1V2::try_from(data)?;
                data.public_key
            }
            WalletVersion::V3R1 | WalletVersion::V3R2 => {
                let data = WalletDataV3::try_from(data)?;
                data.public_key
            }
            WalletVersion::V4R1 | WalletVersion::V4R2 => {
                let data = WalletDataV4::try_from(data)?;
                data.public_key
            }
            WalletVersion::HighloadV2R2 => {
                let data = WalletDataHighloadV2R2::try_from(data)?;
                data.public_key
            }
            _ => {
                //TODO wait WalletVersion derive Debug
                //bail!("can't process given wallet version {:?}", version);
                bail!("can't process given wallet version");
            }
        }
    };
    let pubkey = Ed25519PublicKey::from_bytes(&pubkey_bytes)?;
    let signature_bytes: [u8; 64] = BASE64_STANDARD
        .decode(&proof.signature)?
        .try_into()
        .map_err(|_| anyhow!("expected 64 bit long signature"))?;
    let signature = Ed25519Signature::from_bytes(&signature_bytes)?;
    pubkey.verify(full_msg_hash.as_ref(), &signature)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonlib_core::TonAddress;

    #[test]
    fn test_check_ton_proof() {
        let ton_address = TonAddress::from_hex_str(
            "0:b1481ee8620ebf33b7882fa749654176ef00c7e4cac95ed39f371d5775920814",
        )
        .unwrap();
        let verify_proof_json = r#"{
            "name": "ton_proof",
            "proof": {
                "timestamp": 1730363765,
                "domain": {
                    "length_bytes": 21,
                    "value": "ton-connect.github.io"
                },
                "signature": "BvysFrBS8KgTa3bww9f5paEu6/jZr5jB1JmO6T8nqsLzJqB3hWHiqOG9OezPsiJX3kD9nifMbRhr1xkv37ICCw==",
                "payload": ["","bc1q04uaa0mveqtt4y0sltuxtauhlyl8ctstr5x3hu",""]
            },
            "state_init": "te6cckECFgEAAwQAAgE0ARUBFP8A9KQT9LzyyAsCAgEgAxACAUgEBwLm0AHQ0wMhcbCSXwTgItdJwSCSXwTgAtMfIYIQcGx1Z70ighBkc3RyvbCSXwXgA/pAMCD6RAHIygfL/8nQ7UTQgQFA1yH0BDBcgQEI9ApvoTGzkl8H4AXTP8glghBwbHVnupI4MOMNA4IQZHN0crqSXwbjDQUGAHgB+gD0BDD4J28iMFAKoSG+8uBQghBwbHVngx6xcIAYUATLBSbPFlj6Ahn0AMtpF8sfUmDLPyDJgED7AAYAilAEgQEI9Fkw7UTQgQFA1yDIAc8W9ADJ7VQBcrCOI4IQZHN0coMesXCAGFAFywVQA88WI/oCE8tqyx/LP8mAQPsAkl8D4gIBIAgPAgEgCQ4CAVgKCwA9sp37UTQgQFA1yH0BDACyMoHy//J0AGBAQj0Cm+hMYAIBIAwNABmtznaiaEAga5Drhf/AABmvHfaiaEAQa5DrhY/AABG4yX7UTQ1wsfgAWb0kK29qJoQICga5D6AhhHDUCAhHpJN9KZEM5pA+n/mDeBKAG3gQFImHFZ8xhAT48oMI1xgg0x/TH9MfAvgju/Jk7UTQ0x/TH9P/9ATRUUO68qFRUbryogX5AVQQZPkQ8qP4ACSkyMsfUkDLH1Iwy/9SEPQAye1U+A8B0wchwACfbFGTINdKltMH1AL7AOgw4CHAAeMAIcAC4wABwAORMOMNA6TIyx8Syx/L/xESExQAbtIH+gDU1CL5AAXIygcVy//J0Hd0gBjIywXLAiLPFlAF+gIUy2sSzMzJc/sAyEAUgQEI9FHypwIAcIEBCNcY+gDTP8hUIEeBAQj0UfKnghBub3RlcHSAGMjLBcsCUAbPFlAE+gIUy2oSyx/LP8lz+wACAGyBAQjXGPoA0z8wUiSBAQj0WfKnghBkc3RycHSAGMjLBcsCUAXPFlAD+gITy2rLHxLLP8lz+wAACvQAye1UAFEAAAAAKamjF1M4HQpWKrIhrdY9Ou9RtUmildvf4qB7qOpqgADYbRTiQD9nbsU="
        }"#;
        let verify_proof_data: TonProofData = serde_json::from_str(verify_proof_json).unwrap();
        super::verify_proof(ton_address, verify_proof_data, false).unwrap();
    }
}
