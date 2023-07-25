// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo};
use crate::{
    address::EthereumAddress,
    crypto::{EcdsaRoochSignature, Signature},
    error::RoochError,
};
use anyhow::Result;
use ethers::{
    types::Bytes,
    utils::rlp::{Decodable, Rlp},
};
use fastcrypto::{
    secp256k1::{recoverable::Secp256k1RecoverableSignature, DefaultHash},
    traits::{RecoverableSignature, ToFromBytes},
};
use move_core_types::account_address::AccountAddress;
use moveos_types::{
    h256::H256,
    transaction::{MoveAction, MoveOSTransaction},
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthereumTransaction(pub ethers::core::types::Transaction);

impl EthereumTransaction {
    //This function is just a demo, we should define the Ethereum calldata's MoveAction standard
    pub fn decode_calldata_to_action(&self) -> Result<MoveAction> {
        //Maybe we should use RLP to encode the MoveAction
        bcs::from_bytes(&self.0.input)
            .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
    }

    pub fn convert_eth_signature_to_recoverable_secp256k1_signature(
        &self,
        msg: Bytes,
    ) -> Result<Signature, RoochError> {
        // Normalize the recovery id "v" value to 0 or 1. Ethereum holds recovery id either of 27 or 28.
        let v = if self.0.v.as_u64() == 27 || self.0.v.as_u64() == 28 {
            (self.0.v.as_u64() - 27) as i32
        } else {
            return Err(RoochError::TransactionError(
                "Invalid recovery ID.".to_owned(),
            ));
        };

        // Concatenate "r" and "s" signatures to form the 64-byte "rs" signature
        let mut rsv_signature = [0u8; 65];
        for i in 0..32 {
            rsv_signature[i] = self.0.r.byte(i.try_into().unwrap());
        }
        for i in 0..32 {
            rsv_signature[32 + i] = self.0.s.byte(i.try_into().unwrap());
        }
        // Append the recovery id (v) to the "rsv" signature
        rsv_signature[64] = v as u8;

        // Create the recoverable signature from the rsv signature
        let sig: Secp256k1RecoverableSignature =
            <Secp256k1RecoverableSignature as ToFromBytes>::from_bytes(&rsv_signature).unwrap();
        println!("{}", &sig);
        println!("msg: {:?}", msg.0);
        println!("msg ref: {:?}", msg.0.as_ref());
        // FIXME recover with proper values of r, s, v and message
        // Recover with default Blake2b256 hash to a public key
        let public_key = sig
            .recover_with_hash::<DefaultHash>(msg.0.as_ref())
            .unwrap();

        // Combine the recoverable signature and public key to construct the final signature
        let mut pubkey_and_rsv_signature = Vec::new();
        pubkey_and_rsv_signature.extend_from_slice(&public_key.as_bytes());
        pubkey_and_rsv_signature.extend_from_slice(&rsv_signature);

        // Parse the "pubkey_and_rsv_signature" signature
        // 98 length with 65 bytes recoverable signature and 33 bytes public key, ignore the scheme length
        let signature: Signature =
            <EcdsaRoochSignature as ToFromBytes>::from_bytes(&pubkey_and_rsv_signature)
                .unwrap()
                .into();

        Ok(signature)
    }

    // FIXME implement nonrecoverable signature
    pub fn convert_eth_signature_to_non_recoverable_secp256k1_signature(
        &self,
    ) -> Result<Signature, RoochError> {
        // Concatenate "r" and "s" signatures to form the 64-byte "rs" signature
        let mut rs_signature = [0u8; 64];
        for i in 0..32 {
            rs_signature[i] = self.0.r.byte(i.try_into().unwrap());
        }
        for i in 0..32 {
            rs_signature[32 + i] = self.0.s.byte(i.try_into().unwrap());
        }

        // Create a non-recoverable secp256k1 signature struct without the recovery ID (v)
        let signature: Signature = <EcdsaRoochSignature as ToFromBytes>::from_bytes(&rs_signature)
            .unwrap()
            .into();

        Ok(signature)
    }
}

impl AbstractTransaction for EthereumTransaction {
    fn transaction_type(&self) -> super::TransactionType {
        super::TransactionType::Ethereum
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        let rlp = Rlp::new(bytes);
        let mut tx = ethers::core::types::Transaction::decode(&rlp)?;
        tx.recover_from_mut()?;
        Ok(Self(tx))
    }

    fn encode(&self) -> Vec<u8> {
        self.0.rlp().to_vec()
    }

    fn tx_hash(&self) -> H256 {
        self.0.hash()
    }

    fn authenticator_info(&self) -> AuthenticatorInfo {
        println!("self.0.input: {:?}", &self.0.input);
        let msg = self.0.input.clone();
        AuthenticatorInfo {
            //TODO should change the seqence_number to u256?
            seqence_number: self.0.nonce.as_u64(),
            authenticator: Authenticator::ecdsa(
                self.convert_eth_signature_to_recoverable_secp256k1_signature(msg)
                    .unwrap(),
            ),
        }
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction> {
        let action = self.decode_calldata_to_action()?;
        let tx_ctx = TxContext::new(resolved_sender, self.tx_hash());
        Ok(MoveOSTransaction::new(tx_ctx, action))
    }

    fn sender(&self) -> crate::address::MultiChainAddress {
        EthereumAddress(self.0.from).into()
    }
}
