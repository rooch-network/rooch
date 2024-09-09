// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::str::FromStr;

use crate::address::BitcoinAddress;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::crypto::RoochKeyPair;
use anyhow::{bail, Result};
use bitcoin::bip32::{DerivationPath, Fingerprint};
use bitcoin::key::constants::SCHNORR_PUBLIC_KEY_SIZE;
use bitcoin::key::Secp256k1;
use bitcoin::sighash::{Prevouts, SighashCache};
use bitcoin::taproot::{LeafVersion, TaprootBuilder};
use bitcoin::{Psbt, ScriptBuf, TapLeafHash, TapSighashType, XOnlyPublicKey};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::simple_map::SimpleMap;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    state::MoveState,
    transaction::MoveAction,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("multisign_account");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisignAccountInfo {
    /// The multisign account rooch address
    pub multisign_address: AccountAddress,
    /// The multisign account BitcoinAddress
    pub multisign_bitcoin_address: BitcoinAddress,
    /// The multisign account threshold
    pub threshold: u64,
    /// The public keys of the multisign account
    pub participants: SimpleMap<AccountAddress, ParticipantInfo>,
}

impl MoveStructType for MultisignAccountInfo {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultisignAccountInfo");
}

impl MoveStructState for MultisignAccountInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            BitcoinAddress::type_layout(),
            u64::type_layout(),
            SimpleMap::<AccountAddress, ParticipantInfo>::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub participant_address: AccountAddress,
    pub participant_bitcoin_address: BitcoinAddress,
    pub public_key: Vec<u8>,
}

impl MoveStructType for ParticipantInfo {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ParticipantInfo");
}

impl MoveStructState for ParticipantInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            BitcoinAddress::type_layout(),
            Vec::<u8>::type_layout(),
        ])
    }
}

pub fn generate_multisign_address(
    threshold: usize,
    public_keys: Vec<Vec<u8>>,
) -> Result<BitcoinAddress> {
    let mut x_only_public_keys = public_keys
        .into_iter()
        .map(|pk| {
            let x_only_pk = if pk.len() == SCHNORR_PUBLIC_KEY_SIZE {
                pk
            } else {
                let pubkey = bitcoin::PublicKey::from_slice(&pk)?;
                XOnlyPublicKey::from(pubkey).serialize().to_vec()
            };
            Ok(x_only_pk)
        })
        .collect::<Result<Vec<_>>>()?;

    // Sort public keys to ensure the same script is generated for the same set of keys
    // Note: we sort on the x-only public key bytes
    x_only_public_keys.sort();

    let x_only_public_keys = x_only_public_keys
        .into_iter()
        .map(|pk| XOnlyPublicKey::from_slice(&pk))
        .collect::<Result<Vec<_>, bitcoin::secp256k1::Error>>()?;
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

pub fn update_multisig_psbt(psbt: &mut Psbt, account_info: &MultisignAccountInfo) -> Result<()> {
    let secp = Secp256k1::new();

    let threshold = account_info.threshold as usize;
    let participant_pubkeys = account_info
        .participants
        .values()
        .into_iter()
        .map(|info| Ok(XOnlyPublicKey::from_slice(&info.public_key)?))
        .collect::<Result<Vec<_>>>()?;
    let multisig_script = create_multisig_script(threshold, &participant_pubkeys);

    let mut builder = TaprootBuilder::new();
    builder = builder.add_leaf(0, multisig_script.clone())?;

    let internal_key = participant_pubkeys[0];
    let tap_tree = builder
        .finalize(&secp, internal_key)
        .map_err(|_| anyhow::anyhow!("Failed to finalize taproot tree"))?;

    let tap_leaf_hash = TapLeafHash::from_script(&multisig_script, LeafVersion::TapScript);

    let mut tap_key_origins = BTreeMap::new();
    for pubkey in &participant_pubkeys {
        let default_key_source = (
            Fingerprint::default(),
            DerivationPath::from_str("m").unwrap(),
        );
        tap_key_origins.insert(*pubkey, (vec![tap_leaf_hash], default_key_source));
    }

    for input in psbt.inputs.iter_mut() {
        if let Some(utxo) = &input.witness_utxo {
            let bitcoin_addr = BitcoinAddress::from(utxo.script_pubkey.clone());
            if bitcoin_addr != account_info.multisign_bitcoin_address {
                continue;
            }
            input.tap_internal_key = Some(internal_key);
            input.tap_key_origins = tap_key_origins.clone();
            input.tap_merkle_root = tap_tree.merkle_root();

            let control_block = tap_tree
                .control_block(&(multisig_script.clone(), LeafVersion::TapScript))
                .unwrap();
            input.tap_scripts.insert(
                control_block,
                (multisig_script.clone(), LeafVersion::TapScript),
            );
        }
    }

    Ok(())
}

pub fn sign_taproot_multisig(psbt: &mut Psbt, kp: &RoochKeyPair) -> Result<(), anyhow::Error> {
    let secp = Secp256k1::new();
    let mut sighash_cache = SighashCache::new(&psbt.unsigned_tx);
    let kp = kp.secp256k1_keypair().expect("should have secret key");
    let (our_pubkey, _) = kp.x_only_public_key();

    let spend_utxos = (0..psbt.inputs.len())
        .map(|i| psbt.spend_utxo(i).ok().cloned())
        .collect::<Vec<_>>();

    if !spend_utxos.iter().all(Option::is_some) {
        bail!("Missing spend utxo");
    }

    let all_spend_utxos = spend_utxos.into_iter().flatten().collect::<Vec<_>>();
    let prevouts = Prevouts::All(&all_spend_utxos);
    for (input_index, input) in psbt.inputs.iter_mut().enumerate() {
        if let Some(_tap_internal_key) = input.tap_internal_key {
            let (_control_block, (multisig_script, leaf_version)) =
                input.tap_scripts.iter().next().ok_or_else(|| {
                    anyhow::anyhow!("No tap script found for input {}", input_index)
                })?;

            let tap_leaf_hash = TapLeafHash::from_script(multisig_script, *leaf_version);

            let hash_ty = TapSighashType::Default;

            let sighash = sighash_cache.taproot_script_spend_signature_hash(
                input_index,
                &prevouts,
                tap_leaf_hash,
                hash_ty,
            )?;

            let signature = secp.sign_schnorr(&sighash.into(), &kp);

            input.tap_script_sigs.insert(
                (our_pubkey, tap_leaf_hash),
                bitcoin::taproot::Signature {
                    signature,
                    sighash_type: hash_ty,
                },
            );
        }
    }

    Ok(())
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
    const IS_MULTISIGN_ACCOUNT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("is_multisign_account");
    const PARTICIPANTS_FUNCTION_NAME: &'static IdentStr = ident_str!("participants");
    const THRESHOLD_FUNCTION_NAME: &'static IdentStr = ident_str!("threshold");

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

    pub fn is_multisign_account(&self, multisign_address: AccountAddress) -> Result<bool> {
        let function_call = Self::create_function_call(
            Self::IS_MULTISIGN_ACCOUNT_FUNCTION_NAME,
            vec![],
            vec![multisign_address.to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let is_multisign_account = self
            .caller
            .call_function(&ctx, function_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value).expect("should be a valid bool")
            })?;
        Ok(is_multisign_account)
    }

    pub fn threshold(&self, multisign_address: AccountAddress) -> Result<u64> {
        let function_call = Self::create_function_call(
            Self::THRESHOLD_FUNCTION_NAME,
            vec![],
            vec![multisign_address.to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let threshold = self
            .caller
            .call_function(&ctx, function_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
            })?;
        Ok(threshold)
    }

    pub fn participants(&self, multisign_address: AccountAddress) -> Result<Vec<ParticipantInfo>> {
        let function_call = Self::create_function_call(
            Self::PARTICIPANTS_FUNCTION_NAME,
            vec![],
            vec![multisign_address.to_move_value()],
        );

        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let participants = self
            .caller
            .call_function(&ctx, function_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<Vec<ParticipantInfo>>(&value.value)
                    .expect("should be a valid vector of ParticipantInfo")
            })?;
        Ok(participants)
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

    #[test]
    fn test_multisign_bitcoin_address_from_less_than_eight_pubkeys() {
        let cases = vec![(
            3,
            vec![
                "032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c",
                "039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0",
                "03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61",
                "0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677",
                "03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be",
            ],
            "bc1pwee7tfs79xapsaamzqnnwn8d5w2z3cfzp2v8nhvsyddlyk4l67gqa0x3w5",
        )];
        for (threshold, pubkeys, expected_address) in cases {
            let pubkeys = pubkeys.iter().map(|pk| hex::decode(pk).unwrap()).collect();
            //let expected_address = bitcoin::Address::from_str(expected_address).unwrap();
            test_multisign_address_gen(threshold, pubkeys, Some(expected_address.to_owned()));
        }
    }

    #[test]
    fn test_multisign_bitcoin_address_from_great_than_or_equal_eight_pubkeys() {
        let cases = vec![(
            3,
            vec![
                "032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c",
                "039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0",
                "03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61",
                "0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677",
                "03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be",
                "0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db",
                "028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a",
                "03786e2d94b8aaac17b2846ea908a245ab8b3c9df7ff34be8c75c27beba8e1f579",
            ],
            "bc1p5pmmc8jmfeqx3fx0he3qgylr8q9cmduf43jclgplpk2kcjyrrmzq5tejw6",
        )];
        for (threshold, pubkeys, expected_address) in cases {
            let pubkeys = pubkeys.iter().map(|pk| hex::decode(pk).unwrap()).collect();
            //let expected_address = bitcoin::Address::from_str(expected_address).unwrap();
            test_multisign_address_gen(threshold, pubkeys, Some(expected_address.to_owned()));
        }
    }
}
