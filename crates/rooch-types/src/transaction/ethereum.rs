// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo};
use crate::{
    address::EthereumAddress,
    crypto::{BuiltinScheme, EcdsaRecoverableRoochSignature, Signature},
    error::RoochError,
};
use anyhow::Result;
use ethers::utils::rlp::{Decodable, Rlp};
use fastcrypto::{
    hash::Keccak256,
    secp256k1::recoverable::Secp256k1RecoverableSignature,
    traits::{RecoverableSignature, ToFromBytes},
};
use move_core_types::{account_address::AccountAddress, identifier::Identifier, u256::U256};
use moveos_types::{
    h256::H256,
    transaction::{MoveAction, MoveOSTransaction},
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthereumTransaction(pub ethers::core::types::Transaction);

impl EthereumTransaction {
    // This function is just a demo, we should define the Ethereum calldata's MoveAction standard
    pub fn decode_calldata_to_action(&self) -> Result<MoveAction> {
        if self.0.input.is_empty() {
            //let account_address = self.rpc_service.resolve_address(self.sender()).await?;

            let test = TxContext::random_for_testing_only();
            let serialized = test.to_bytes();

            // Handle the transfer gas coin function call only in ethereum
            let payload = MoveAction::new_function_call(
                moveos_types::move_types::FunctionId::new(
                    move_core_types::language_storage::ModuleId::new(
                        AccountAddress::from_hex_literal("0x3").unwrap(),
                        Identifier::new("coin").unwrap(),
                    ),
                    Identifier::new("transfer").unwrap(),
                ),
                vec![move_core_types::language_storage::TypeTag::Struct(
                    Box::new(move_core_types::language_storage::StructTag {
                        address: AccountAddress::from_hex_literal("0x3").unwrap(),
                        module: Identifier::new("gas_coin").unwrap(),
                        name: Identifier::new("GasCoin").unwrap(),
                        type_params: vec![],
                    }),
                )],
                vec![
                    serialized,
                    move_core_types::value::MoveValue::Signer(AccountAddress::random())
                        .simple_serialize()
                        .unwrap(),
                    move_core_types::value::MoveValue::Address(AccountAddress::random())
                        .simple_serialize()
                        .unwrap(),
                    move_core_types::value::MoveValue::U256(U256::max_value())
                        .simple_serialize()
                        .unwrap(),
                ],
            );

            Ok(payload)
        } else {
            // Maybe we should use RLP to encode the MoveAction
            bcs::from_bytes(&self.0.input)
                .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
        }
    }

    // Calculate the "recovery byte": The recovery ID (v) contains information about the network and the signature type.
    fn normalize_recovery_id(v: u64) -> u8 {
        match v {
            0 => 0,
            1 => 1,
            27 => 0,
            28 => 1,
            v if v >= 35 => ((v - 1) % 2) as _,
            _ => 4,
        }
    }

    pub fn convert_eth_transaction_signature_to_rooch_signature(
        &self,
    ) -> Result<Signature, RoochError> {
        let r = self.0.r;
        let s = self.0.s;
        let v = self.0.v.as_u64();

        let recovery_id = Self::normalize_recovery_id(v);

        // Prepare the signed message (RLP encoding of the transaction)
        let message = self.tx_hash().to_fixed_bytes();

        // Convert `U256` values `r` and `s` to arrays of `u8`
        let mut r_bytes = [0u8; 32];
        r.to_big_endian(&mut r_bytes);
        let mut s_bytes = [0u8; 32];
        s.to_big_endian(&mut s_bytes);

        // Create a new array to store the 65-byte "rsv" signature
        let mut rsv_signature = [0u8; 65];
        rsv_signature[..32].copy_from_slice(&r_bytes);
        rsv_signature[32..64].copy_from_slice(&s_bytes);
        rsv_signature[64] = recovery_id;

        // Create the recoverable signature from the rsv signature
        let recoverable_signature: Secp256k1RecoverableSignature =
            <Secp256k1RecoverableSignature as ToFromBytes>::from_bytes(&rsv_signature)
                .expect("Invalid signature");

        // Recover with Keccak256 hash to a public key
        let public_key = recoverable_signature
            .recover_with_hash::<Keccak256>(&message)
            .expect("Failed to recover public key");

        // Combine the scheme, recoverable signature and public key as the following order to construct the final signature
        // 1. The Rooch crypto scheme
        // 2. The RSV signature
        // 3. The public key
        let mut scheme_rsv_signature_pubkey = Vec::new();
        scheme_rsv_signature_pubkey.push(BuiltinScheme::EcdsaRecoverable.flag());
        scheme_rsv_signature_pubkey.extend_from_slice(&rsv_signature);
        scheme_rsv_signature_pubkey.extend_from_slice(public_key.as_bytes());

        // Parse the "scheme_rsv_signature_pubkey" signature
        // 99 length with 1 byte scheme following with 65 bytes recoverable signature and 33 bytes public key
        let signature: Signature = <EcdsaRecoverableRoochSignature as ToFromBytes>::from_bytes(
            &scheme_rsv_signature_pubkey,
        )
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

    fn authenticator_info(&self) -> Result<AuthenticatorInfo> {
        let chain_id = self.0.chain_id.ok_or(RoochError::InvalidChainID)?.as_u64();
        let authenticator = Authenticator::ecdsa_recoverable(
            self.convert_eth_transaction_signature_to_rooch_signature()?,
        );
        Ok(AuthenticatorInfo::new(chain_id, authenticator))
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction> {
        let action = self.decode_calldata_to_action()?;
        let sequence_number = self.0.nonce.as_u64();
        let gas = self.0.gas.as_u64();
        let tx_ctx = TxContext::new(resolved_sender, sequence_number, gas, self.tx_hash());
        Ok(MoveOSTransaction::new(tx_ctx, action))
    }

    fn sender(&self) -> crate::address::MultiChainAddress {
        EthereumAddress(self.0.from).into()
    }
}
