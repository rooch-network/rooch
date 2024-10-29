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
pub struct TonProof {
    pub domain: TonDomain,
    pub payload: String,
    pub signature: String,
    pub state_init: String,
    pub timestamp: u64,
}

#[derive(Deserialize)]
pub struct TonDomain {
    #[serde(rename = "lengthBytes")]
    pub length_bytes: u64,
    pub value: String,
}

pub fn check_ton_proof(address: TonAddress, proof: TonProof) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // check ton proof expiration
    if now > proof.timestamp + PROOF_TTL {
        bail!("ton proof has been expired");
    }

    if proof.domain.length_bytes != proof.domain.value.len() as u64 {
        bail!(
            "domain length mismatched against provided length bytes of {}",
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
        let bytes = BASE64_STANDARD.decode(&proof.state_init)?;
        let boc = BagOfCells::parse(&bytes)?;
        let hash: [u8; 32] = boc.single_root()?.cell_hash();

        if hash != address.hash_part {
            return Err(anyhow!("wrong address in state_init"));
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

    #[test]
    fn test_check_ton_proof() {}
}
