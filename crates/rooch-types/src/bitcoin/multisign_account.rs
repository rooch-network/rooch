// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::BitcoinAddress;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use bitcoin::key::constants::SCHNORR_PUBLIC_KEY_SIZE;
use bitcoin::key::Secp256k1;
use bitcoin::taproot::TaprootBuilder;
use bitcoin::{ScriptBuf, XOnlyPublicKey};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    state::MoveState,
    transaction::MoveAction,
};

pub const MODULE_NAME: &IdentStr = ident_str!("multisign_account");

pub fn generate_multisign_address(
    threshold: usize,
    public_keys: Vec<Vec<u8>>,
) -> Result<BitcoinAddress> {
    let mut x_only_public_keys = public_keys
        .into_iter()
        .map(|pk| {
            let x_only_pk = if pk.len() == SCHNORR_PUBLIC_KEY_SIZE {
                XOnlyPublicKey::from_slice(&pk)?
            } else {
                let pubkey = bitcoin::PublicKey::from_slice(&pk)?;
                XOnlyPublicKey::from(pubkey)
            };
            Ok(x_only_pk)
        })
        .collect::<Result<Vec<_>>>()?;

    // Sort public keys to ensure the same script is generated for the same set of keys
    // Note: we sort on the x-only public key
    x_only_public_keys.sort();

    let multisig_script = create_multisig_script(threshold, &x_only_public_keys);

    let builder = TaprootBuilder::new().add_leaf(0, multisig_script)?;
    let secp = Secp256k1::verification_only();
    //Use the first public key after sorted as the internal key
    let internal_key = x_only_public_keys[0];

    let spend_info = builder.finalize(&secp, internal_key).unwrap();

    let address = bitcoin::Address::p2tr(
        &secp,
        internal_key,
        spend_info.merkle_root(),
        bitcoin::Network::Bitcoin,
    );
    Ok(address.into())
}

/// Create a multisig script, the caller should ensure the public keys are sorted
fn create_multisig_script(threshold: usize, public_keys: &Vec<XOnlyPublicKey>) -> ScriptBuf {
    let mut builder = bitcoin::script::Builder::new();

    for pubkey in public_keys {
        if builder.is_empty() {
            builder = builder.push_x_only_key(pubkey);
            builder = builder.push_opcode(bitcoin::opcodes::all::OP_CHECKSIG);
        } else {
            builder = builder.push_x_only_key(pubkey);
            builder = builder.push_opcode(bitcoin::opcodes::all::OP_CHECKSIGADD);
        }
    }
    builder = builder.push_int(threshold as i64);
    builder = builder.push_opcode(bitcoin::opcodes::all::OP_GREATERTHANOREQUAL);

    builder.into_script()
}

/// Rust bindings for multisign_acount module
pub struct MultisignAccountModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> MultisignAccountModule<'a> {
    const INITIALIZE_MULTISIG_ACCOUNT_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("initialize_multisig_account_entry");
    const GENERATE_MULTISIGN_ADDRESS_FUNCTION_NAME: &'static IdentStr =
        ident_str!("generate_multisign_address");
    const IS_PARTICIPANT_FUNCTION_NAME: &'static IdentStr = ident_str!("is_participant");

    pub fn initialize_multisig_account_action(
        threshold: u64,
        public_keys: Vec<Vec<u8>>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::INITIALIZE_MULTISIG_ACCOUNT_ENTRY_FUNCTION_NAME,
            vec![],
            vec![threshold.to_move_value(), public_keys.to_move_value()],
        )
    }

    pub fn generate_multisign_address(
        &self,
        threshold: u64,
        public_keys: Vec<Vec<u8>>,
    ) -> Result<BitcoinAddress> {
        let function_call = Self::create_function_call(
            Self::GENERATE_MULTISIGN_ADDRESS_FUNCTION_NAME,
            vec![],
            vec![threshold.to_move_value(), public_keys.to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let address = self
            .caller
            .call_function(&ctx, function_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<BitcoinAddress>(&value.value)
                    .expect("should be a valid BitcoinAddress")
            })?;
        Ok(address)
    }

    pub fn is_participant(
        &self,
        multisign_address: AccountAddress,
        participant_address: AccountAddress,
    ) -> Result<bool> {
        let function_call = Self::create_function_call(
            Self::IS_PARTICIPANT_FUNCTION_NAME,
            vec![],
            vec![
                multisign_address.to_move_value(),
                participant_address.to_move_value(),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let is_participant = self
            .caller
            .call_function(&ctx, function_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value).expect("should be a valid bool")
            })?;
        Ok(is_participant)
    }
}

impl<'a> ModuleBinding<'a> for MultisignAccountModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::RoochKeyPair;
    use std::str::FromStr;

    fn test_multisign_address_gen(
        threshold: usize,
        pubkeys: Vec<Vec<u8>>,
        expected_address: Option<String>,
    ) {
        let address = generate_multisign_address(threshold, pubkeys).unwrap();
        if let Some(expected_address) = expected_address {
            assert_eq!(
                address,
                BitcoinAddress::from_str(&expected_address).unwrap()
            );
        }
    }

    #[test]
    fn test_multisign_address_random() {
        let kp1 = RoochKeyPair::generate_secp256k1();
        let kp2 = RoochKeyPair::generate_secp256k1();
        let kp3 = RoochKeyPair::generate_secp256k1();
        let pubkeys = vec![
            kp1.bitcoin_public_key().unwrap(),
            kp2.bitcoin_public_key().unwrap(),
            kp3.bitcoin_public_key().unwrap(),
        ];
        let pubkeys = pubkeys.into_iter().map(|pk| pk.to_bytes()).collect();
        test_multisign_address_gen(2, pubkeys, None);
    }

    #[test]
    fn test_multisign_address_generate_with_fix_pubkey() {
        let cases = vec![(
            2,
            vec![
                "0308839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4",
                "0338121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752",
                "03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579",
            ],
            "tb1phldgaz7jzshk4zw60hvveeac498jt57dst25kuhuut96dkl6kvcskvg57y",
        )];
        for (threshold, pubkeys, expected_address) in cases {
            let pubkeys = pubkeys.iter().map(|pk| hex::decode(pk).unwrap()).collect();
            //let expected_address = bitcoin::Address::from_str(expected_address).unwrap();
            test_multisign_address_gen(threshold, pubkeys, Some(expected_address.to_owned()));
        }
    }

    #[test]
    fn test_create_multisig_script() {
        let cases = vec![
            (
                2,
                vec![
                    "0308839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4",
"0338121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752",
"03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579"],
                "2008839c624d3da34ae240086f60196409d619f285365cc3498fdd3a90b72599e4ac2038121decf4ea2dbfd2ad1fe05a32a67448e78bf97a18bc107b4da177c27af752ba20786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579ba52a2"
            )
        ];
        for (threshold, pubkeys, expected_script) in cases {
            let pubkeys = pubkeys
                .iter()
                .map(|pk| XOnlyPublicKey::from(bitcoin::PublicKey::from_str(pk).unwrap()))
                .collect::<Vec<_>>();
            let expected_script =
                bitcoin::ScriptBuf::from_bytes(hex::decode(expected_script).unwrap());
            let script = create_multisig_script(threshold, &pubkeys);
            //println!("script: {:?}", script.to_hex_string());
            assert_eq!(script, expected_script);
        }
    }
}
