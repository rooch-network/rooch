// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types;
use crate::{addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};
use anyhow::{anyhow, Result};
use bitcoin::{
    key::Secp256k1,
    opcodes::all::*,
    script::Builder,
    taproot::{LeafVersion, TaprootBuilder, TaprootBuilderError, TaprootSpendInfo},
    Address, Amount, Network, PublicKey, Script, ScriptBuf, TapLeafHash, TxOut, XOnlyPublicKey,
};
use bitcoin::{Transaction, Txid};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("bbn");

// TagLen length of tag prefix identifying staking transactions
pub const TAG_LEN: usize = 4;
// V0OpReturnDataSize 4 bytes tag + 1 byte version + 32 bytes staker public key + 32 bytes finality provider public key + 2 bytes staking time
pub const V0_OP_RETURN_DATA_SIZE: usize = 71;

//https://github.com/babylonlabs-io/networks/blob/main/bbn-1/parameters/global-params.json
// {
//     "version": 1,
//     "activation_height": 864790,
//     "cap_height": 864799,
//     "tag": "62626e31",
//     "covenant_pks": [
//         "03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
//         "034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
//         "0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
//         "02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
//         "038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
//         "03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
//         "03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
//         "03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
//         "03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
//     ],
//     "covenant_quorum": 6,
//     "unbonding_time": 1008,
//     "unbonding_fee": 32000,
//     "max_staking_amount": 50000000000,
//     "min_staking_amount": 500000,
//     "max_staking_time": 64000,
//     "min_staking_time": 64000,
//     "confirmation_depth": 10
// }
pub static BBN_GLOBAL_PARAM_BBN1: Lazy<BBNGlobalParamV1> = Lazy::new(|| BBNGlobalParamV1 {
    version: 1,
    activation_height: 864790,
    cap_height: 864799,
    tag: hex::decode("62626e31").unwrap(),
    covenant_pks: vec![
        hex::decode("d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa").unwrap(),
        hex::decode("4b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa").unwrap(),
        hex::decode("23b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1").unwrap(),
        hex::decode("d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae").unwrap(),
        hex::decode("8242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7").unwrap(),
        hex::decode("e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c").unwrap(),
        hex::decode("cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204").unwrap(),
        hex::decode("f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0").unwrap(),
        hex::decode("de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c").unwrap(),
    ],
    covenant_quorum: 6,
    unbonding_time: 1008,
    unbonding_fee: 32000,
    max_staking_amount: 50000000000,
    min_staking_amount: 500000,
    min_staking_time: 64000,
    max_staking_time: 64000,
    confirmation_depth: 10,
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNGlobalParams {
    pub max_version: u64,
}

impl MoveStructType for BBNGlobalParams {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNGlobalParams");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNGlobalParams {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64])
    }
}

impl BBNGlobalParams {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNGlobalParamV1 {
    pub version: u64,
    pub activation_height: u64,
    pub cap_height: u64,
    pub tag: Vec<u8>,
    pub covenant_pks: Vec<Vec<u8>>,
    pub covenant_quorum: u32,
    pub unbonding_time: u16,
    pub unbonding_fee: u64,
    pub max_staking_amount: u64,
    pub min_staking_amount: u64,
    pub min_staking_time: u16,
    pub max_staking_time: u16,
    pub confirmation_depth: u16,
}

impl MoveStructType for BBNGlobalParamV1 {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNGlobalParamV1");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNGlobalParamV1 {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::Vector(Box::new(
                MoveTypeLayout::U8,
            )))),
            MoveTypeLayout::U32,
            MoveTypeLayout::U16,
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::U16,
            MoveTypeLayout::U16,
            MoveTypeLayout::U16,
        ])
    }
}

impl BBNGlobalParamV1 {
    pub fn get_covenant_pks(&self) -> Vec<XOnlyPublicKey> {
        self.covenant_pks
            .iter()
            .map(|pk| XOnlyPublicKey::from_slice(pk).unwrap())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNStakeSeal {
    /// The stake transaction block height
    pub block_height: u64,
    /// The stake transaction hash
    pub txid: AccountAddress,
    /// The stake utxo output index
    pub staking_output_index: u32,
    pub tag: Vec<u8>,
    pub staker_pub_key: Vec<u8>,
    pub finality_provider_pub_key: Vec<u8>,
    /// The stake time in block count
    pub staking_time: u16,
    /// The stake value amount in satoshi
    pub staking_value: u64,
}

impl MoveStructType for BBNStakeSeal {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNStakeSeal");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNStakeSeal {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            MoveTypeLayout::U32,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U16,
            MoveTypeLayout::U64,
        ])
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BBNV0OpReturnData {
    pub tag: Vec<u8>,
    pub version: u8,
    pub staker_pub_key: Vec<u8>,
    pub finality_provider_pub_key: Vec<u8>,
    pub staking_time: u16,
}

impl fmt::Debug for BBNV0OpReturnData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BBNV0OpReturnData")
            .field("tag", &hex::encode(&self.tag))
            .field("version", &self.version)
            .field("staker_pub_key", &hex::encode(&self.staker_pub_key))
            .field(
                "finality_provider_pub_key",
                &hex::encode(&self.finality_provider_pub_key),
            )
            .field("staking_time", &self.staking_time)
            .finish()
    }
}

impl MoveStructType for BBNV0OpReturnData {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNV0OpReturnData");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNV0OpReturnData {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U8,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U16,
        ])
    }
}

impl BBNV0OpReturnData {
    pub fn parse_from_script(script: &ScriptBuf) -> Result<Self> {
        parse_bbn_op_return_data(script)
    }

    pub fn staker_pub_key(&self) -> Result<XOnlyPublicKey> {
        Ok(XOnlyPublicKey::from_slice(&self.staker_pub_key)?)
    }

    pub fn finality_provider_pub_key(&self) -> Result<XOnlyPublicKey> {
        Ok(XOnlyPublicKey::from_slice(&self.finality_provider_pub_key)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNV0OpReturnOutput {
    pub op_return_output_idx: u32,
    pub op_return_data: BBNV0OpReturnData,
}

impl MoveStructType for BBNV0OpReturnOutput {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNV0OpReturnOutput");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNV0OpReturnOutput {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U32, BBNV0OpReturnData::type_layout()])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNStakingEvent {
    pub block_height: u64,
    pub txid: AccountAddress,
    /// BBNStakeSeal object id
    pub stake_object_id: ObjectID,
}

impl MoveStructType for BBNStakingEvent {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNStakingEvent");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNStakingEvent {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            ObjectID::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNStakingFailedEvent {
    pub block_height: u64,
    pub txid: AccountAddress,
    pub error: String,
}

impl MoveStructType for BBNStakingFailedEvent {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNStakingFailedEvent");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNStakingFailedEvent {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }
}

/// Rust bindings for BitcoinMove bitcoin module
pub struct BBNModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BBNModule<'a> {
    pub const TRY_GET_BBN_OP_RETURN_OUTPUT_FROM_TX_BYTES_FUNCTION_NAME: &'static IdentStr =
        ident_str!("try_get_bbn_op_return_ouput_from_tx_bytes");
    pub const TRY_GET_BBN_STAKING_OUTPUT_FROM_TX_BYTES: &'static IdentStr =
        ident_str!("try_get_bbn_staking_output_from_tx_bytes");
    pub const IS_POSSIBLE_BBN_TX_FUNCTION_NAME: &'static IdentStr =
        ident_str!("is_possible_bbn_tx");
    pub const PROCESS_BBN_TX_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("process_bbn_tx_entry");

    pub fn try_get_bbn_op_return_ouput(
        &self,
        tx: Transaction,
    ) -> Result<Option<BBNV0OpReturnOutput>> {
        let rooch_btc_tx = types::Transaction::from(tx);
        let tx_bytes = bcs::to_bytes(&rooch_btc_tx).expect("should be a valid transaction");
        let call = Self::create_function_call(
            Self::TRY_GET_BBN_OP_RETURN_OUTPUT_FROM_TX_BYTES_FUNCTION_NAME,
            vec![],
            vec![tx_bytes.to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let result = self.caller.call_function(&ctx, call)?;
        let bbn_op_return_output_opt = result
            .into_result()
            .map(|mut values| {
                let bbn_op_return_output = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<BBNV0OpReturnOutput>>(&bbn_op_return_output.value)
                    .expect("should be a valid BBNOpReturnOutput")
            })
            .map_err(|e| anyhow::anyhow!("Failed to get bbn op return data: {:?}", e))?;
        Ok(bbn_op_return_output_opt.into())
    }

    pub fn try_get_bbn_staking_output(
        &self,
        tx: Transaction,
        staking_output_pk_script: &ScriptBuf,
    ) -> Result<Option<u32>> {
        let rooch_btc_tx = types::Transaction::from(tx);
        let tx_bytes = bcs::to_bytes(&rooch_btc_tx).expect("should be a valid transaction");
        let call = Self::create_function_call(
            Self::TRY_GET_BBN_STAKING_OUTPUT_FROM_TX_BYTES,
            vec![],
            vec![
                tx_bytes.to_move_value(),
                staking_output_pk_script.as_bytes().to_vec().to_move_value(),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let result = self.caller.call_function(&ctx, call)?;
        let bbn_staking_output_opt = result
            .into_result()
            .map(|mut values| {
                let bbn_staking_output = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<u32>>(&bbn_staking_output.value)
                    .expect("should be a valid MoveOption<u32>")
            })
            .map_err(|e| anyhow::anyhow!("Failed to get bbn op return data: {:?}", e))?;
        Ok(bbn_staking_output_opt.into())
    }

    pub fn is_possible_bbn_tx(&self, txid: Txid) -> Result<bool> {
        let call = Self::create_function_call(
            Self::IS_POSSIBLE_BBN_TX_FUNCTION_NAME,
            vec![],
            vec![txid.into_address().to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let is_bbn_tx = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let is_bbn_tx_return = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&is_bbn_tx_return.value).expect("should be a valid bool")
            })
            .map_err(|e| anyhow::anyhow!("Failed to get is bbn tx: {:?}", e))?;
        Ok(is_bbn_tx)
    }

    pub fn create_process_bbn_tx_entry_call(txid: Txid) -> Result<FunctionCall> {
        Ok(Self::create_function_call(
            Self::PROCESS_BBN_TX_ENTRY_FUNCTION_NAME,
            vec![],
            vec![txid.into_address().to_move_value()],
        ))
    }
}

impl<'a> ModuleBinding<'a> for BBNModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

// ================================ BBN Transaction Parsing ================================

//https://github.com/babylonlabs-io/babylon/blob/07e4437f993d32a2f24d33531cf7de7ac87fd0d9/btcstaking/types.go#L192

/// Point with unknown discrete logarithm defined in: https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
/// using it as internal public key effectively disables taproot key spends
const UNSPENDABLE_KEY_PATH: &str =
    "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

static UNSPENDABLE_KEY_PATH_KEY: Lazy<XOnlyPublicKey> =
    Lazy::new(|| XOnlyPublicKey::from(PublicKey::from_str(UNSPENDABLE_KEY_PATH).unwrap()));

#[derive(Debug, Clone)]
pub struct BBNParsedV0StakingTx {
    pub staking_output: TxOut,
    pub staking_output_idx: u32,
    pub op_return_output: TxOut,
    pub op_return_output_idx: u32,
    pub op_return_data: BBNV0OpReturnData,
}

impl BBNParsedV0StakingTx {
    pub fn parse_from_tx(
        tx: &Transaction,
        expected_tag: &[u8],
        covenant_keys: &[XOnlyPublicKey],
        covenant_quorum: u32,
    ) -> Result<Self> {
        if tx.output.len() < 2 {
            return Err(anyhow!("Transaction has insufficient outputs"));
        }

        if covenant_keys.is_empty() {
            return Err(anyhow!("No covenant keys specified"));
        }

        if covenant_quorum > covenant_keys.len() as u32 {
            return Err(anyhow!(
                "Covenant quorum is greater than the number of covenant keys"
            ));
        }

        let BBNV0OpReturnOutput {
            op_return_output_idx,
            op_return_data,
        } = try_get_bbn_op_return_ouput(&tx.output)
            .ok_or_else(|| anyhow!("No valid staking transaction OP_RETURN output found"))?;

        if op_return_data.version != 0 {
            return Err(anyhow!("Unsupported BBN transaction version"));
        }

        if op_return_data.tag != expected_tag {
            return Err(anyhow!(
                "Unexpected tag, expected {}, got {}",
                hex::encode(expected_tag),
                hex::encode(&op_return_data.tag)
            ));
        }

        let staking_info = BBNStakingInfo::build_staking_info(
            &op_return_data.staker_pub_key()?,
            &[op_return_data.finality_provider_pub_key()?],
            covenant_keys,
            covenant_quorum,
            op_return_data.staking_time,
            0,
        )?;

        let (staking_output, staking_output_idx) =
            try_get_bbn_staking_output(&tx.output, &staking_info.staking_output.script_pubkey)
                .ok_or_else(|| {
                    anyhow!("Staking output not found in potential staking transaction")
                })?;

        Ok(Self {
            staking_output,
            staking_output_idx,
            op_return_output: tx.output[op_return_output_idx as usize].clone(),
            op_return_output_idx,
            op_return_data,
        })
    }

    pub fn is_possible_staking_tx(tx: &Transaction, expected_tag: &[u8]) -> bool {
        if tx.output.len() < 2 {
            return false;
        }

        let mut possible_staking_tx = false;
        for output in &tx.output {
            if let Ok(data) = get_v0_op_return_bytes(output) {
                if data.len() < TAG_LEN + 1 {
                    continue;
                }

                if &data[..TAG_LEN] != expected_tag {
                    continue;
                }

                if data[TAG_LEN] != 0 {
                    continue;
                }

                if possible_staking_tx {
                    return false;
                }

                possible_staking_tx = true;
            }
        }

        possible_staking_tx
    }
}

#[derive(Debug, Clone)]
pub struct TaprootScriptHolder {
    internal_pub_key: XOnlyPublicKey,
    script_tree: TaprootSpendInfo,
}

impl TaprootScriptHolder {
    pub fn new<C>(
        secp: &Secp256k1<C>,
        internal_pub_key: XOnlyPublicKey,
        time_lock_script: ScriptBuf,
        unbonding_path_script: ScriptBuf,
        slashing_path_script: ScriptBuf,
    ) -> Result<Self>
    where
        C: bitcoin::secp256k1::Verification,
    {
        let taproot_builder = assemble_taproot_script_tree(
            time_lock_script,
            unbonding_path_script,
            slashing_path_script,
        )
        .map_err(|e| anyhow!("Failed to build taproot tree {:?}", e))?;
        let taproot_spend_info = taproot_builder
            .finalize(secp, internal_pub_key)
            .map_err(|e| anyhow!("Failed to finalize taproot tree: {:?}", e))?;

        Ok(Self {
            internal_pub_key,
            script_tree: taproot_spend_info,
        })
    }

    pub fn taproot_pk_script(&self) -> ScriptBuf {
        let output_key = self.script_tree.output_key();
        let address = Address::p2tr_tweaked(output_key, Network::Bitcoin);
        address.script_pubkey()
    }

    pub fn internal_pub_key(&self) -> XOnlyPublicKey {
        self.internal_pub_key
    }
}

#[derive(Debug, Clone)]
pub struct BBNStakingInfo {
    pub staking_output: TxOut,
    pub script_holder: TaprootScriptHolder,
    pub time_lock_path_leaf_hash: TapLeafHash,
    pub unbonding_path_leaf_hash: TapLeafHash,
    pub slashing_path_leaf_hash: TapLeafHash,
}

impl BBNStakingInfo {
    pub fn build_staking_info(
        staker_key: &XOnlyPublicKey,
        fp_keys: &[XOnlyPublicKey],
        covenant_keys: &[XOnlyPublicKey],
        covenant_quorum: u32,
        staking_time: u16,
        staking_amount: u64,
    ) -> Result<Self> {
        let unspendable_key_path_key = *UNSPENDABLE_KEY_PATH_KEY;
        let script_paths = BBNScriptPaths::build(
            staker_key,
            fp_keys,
            covenant_keys,
            covenant_quorum,
            staking_time,
        )?;

        let taproot_script_holder = TaprootScriptHolder::new(
            &Secp256k1::new(),
            unspendable_key_path_key,
            script_paths.time_lock_path_script.clone(),
            script_paths.unbonding_path_script.clone(),
            script_paths.slashing_path_script.clone(),
        )?;
        let script_pubkey = taproot_script_holder.taproot_pk_script();

        let staking_output = TxOut {
            value: Amount::from_sat(staking_amount),
            script_pubkey,
        };

        Ok(Self {
            staking_output,
            script_holder: taproot_script_holder,
            time_lock_path_leaf_hash: script_paths.time_lock_path_script_hash(),
            unbonding_path_leaf_hash: script_paths.unbonding_path_script_hash(),
            slashing_path_leaf_hash: script_paths.slashing_path_script_hash(),
        })
    }
}

#[derive(Debug, Clone)]
struct BBNScriptPaths {
    time_lock_path_script: ScriptBuf,
    unbonding_path_script: ScriptBuf,
    slashing_path_script: ScriptBuf,
}

impl BBNScriptPaths {
    pub fn build(
        staker_key: &XOnlyPublicKey,
        fp_keys: &[XOnlyPublicKey],
        covenant_keys: &[XOnlyPublicKey],
        covenant_quorum: u32,
        lock_time: u16,
    ) -> Result<Self> {
        let time_lock_path_script = build_time_lock_script(staker_key, lock_time);

        let covenant_multisig_script =
            build_multi_sig_script(covenant_keys, covenant_quorum, false)?;

        let staker_sig_script = build_single_key_sig_script(staker_key, true);

        let fp_multisig_script = build_multi_sig_script(fp_keys, 1, true)?;

        let unbonding_path_script = aggregate_scripts(vec![
            staker_sig_script.clone(),
            covenant_multisig_script.clone(),
        ]);

        let slashing_path_script = aggregate_scripts(vec![
            staker_sig_script.clone(),
            fp_multisig_script.clone(),
            covenant_multisig_script.clone(),
        ]);

        Ok(BBNScriptPaths {
            time_lock_path_script,
            unbonding_path_script,
            slashing_path_script,
        })
    }

    fn time_lock_path_script_hash(&self) -> TapLeafHash {
        TapLeafHash::from_script(&self.time_lock_path_script, LeafVersion::TapScript)
    }

    fn unbonding_path_script_hash(&self) -> TapLeafHash {
        TapLeafHash::from_script(&self.unbonding_path_script, LeafVersion::TapScript)
    }

    fn slashing_path_script_hash(&self) -> TapLeafHash {
        TapLeafHash::from_script(&self.slashing_path_script, LeafVersion::TapScript)
    }
}

// Only the holder of the private key for the given public key can spend after the relative lock time
// Script: <StakerPk> OP_CHECKSIGVERIFY <stakingTime> OP_CHECKSEQUENCEVERIFY
fn build_time_lock_script(pub_key: &XOnlyPublicKey, lock_time: u16) -> ScriptBuf {
    Builder::new()
        .push_x_only_key(pub_key)
        .push_opcode(OP_CHECKSIGVERIFY)
        .push_int(lock_time as i64)
        //https://github.com/bitcoin/bips/blob/master/bip-0112.mediawiki
        .push_opcode(OP_CSV)
        .into_script()
}

// Only the holder of the private key for the given public key can spend
// Script: <pubKey> OP_CHECKSIGVERIFY
fn build_single_key_sig_script(pub_key: &XOnlyPublicKey, with_verify: bool) -> ScriptBuf {
    let builder = Builder::new().push_x_only_key(pub_key);
    let builder = if with_verify {
        builder.push_opcode(OP_CHECKSIGVERIFY)
    } else {
        builder.push_opcode(OP_CHECKSIG)
    };
    builder.into_script()
}

//Make sure the keys are sorted according to the babylon chain
//https://github.com/babylonlabs-io/babylon/blob/07e4437f993d32a2f24d33531cf7de7ac87fd0d9/btcstaking/scripts_utils.go#L42-L54
fn sort_keys(mut keys: Vec<XOnlyPublicKey>) -> Vec<XOnlyPublicKey> {
    keys.sort();
    keys
}

fn build_multi_sig_script(
    keys: &[XOnlyPublicKey],
    threshold: u32,
    with_verify: bool,
) -> Result<ScriptBuf> {
    if keys.is_empty() {
        return Err(anyhow::anyhow!("no keys provided"));
    }

    if threshold > keys.len() as u32 {
        return Err(anyhow::anyhow!(
            "required number of valid signers is greater than number of provided keys"
        ));
    }

    if keys.len() == 1 {
        return Ok(build_single_key_sig_script(&keys[0], with_verify));
    }
    let keys = sort_keys(keys.to_vec());
    let mut builder = Builder::new();

    for (i, key) in keys.iter().enumerate() {
        builder = builder.push_x_only_key(key);
        if i == 0 {
            builder = builder.push_opcode(OP_CHECKSIG);
        } else {
            builder = builder.push_opcode(OP_CHECKSIGADD);
        }
    }

    builder = builder.push_int(threshold as i64);
    builder = builder.push_opcode(if with_verify {
        OP_NUMEQUALVERIFY
    } else {
        OP_NUMEQUAL
    });

    Ok(builder.into_script())
}

fn aggregate_scripts(scripts: Vec<ScriptBuf>) -> ScriptBuf {
    let mut final_script = Vec::new();
    for script in scripts {
        final_script.extend_from_slice(script.as_bytes());
    }
    ScriptBuf::from(final_script)
}

fn get_v0_op_return_bytes(output: &TxOut) -> Result<Vec<u8>> {
    if output.script_pubkey.is_op_return() {
        let script_bytes = output.script_pubkey.as_bytes();
        // We are adding `+2` as each op return has additional 2 for:
        // 1. OP_RETURN opcode - which signalizes that data is provably unspendable
        // 2. OP_DATA_71 opcode - which pushes 71 bytes of data to the stack
        if script_bytes.len() != V0_OP_RETURN_DATA_SIZE + 2 {
            return Err(anyhow::anyhow!("Invalid OP_RETURN data length"));
        }
        Ok(script_bytes[2..].to_vec())
    } else {
        Err(anyhow::anyhow!("Output is not an OP_RETURN"))
    }
}

fn parse_bbn_op_return_data(script: &Script) -> Result<BBNV0OpReturnData> {
    let script_bytes = script.as_bytes();
    let script_len = script_bytes.len();

    if script_len != 73
        || script_bytes[0] != OP_RETURN.to_u8()
        || script_bytes[1] != OP_PUSHBYTES_71.to_u8()
    {
        return Err(anyhow::anyhow!("Invalid BBN OP_RETURN script"));
    }

    let tag = script_bytes[2..6].to_vec();
    let version = script_bytes[6];

    if version != 0u8 {
        return Err(anyhow::anyhow!("Unsupported BBN OP_RETURN version"));
    }

    let staker_pub_key = script_bytes[7..39].to_vec();
    let finality_provider_pub_key = script_bytes[39..71].to_vec();
    let staking_time = u16::from_be_bytes([script_bytes[71], script_bytes[72]]);

    Ok(BBNV0OpReturnData {
        tag,
        version,
        staker_pub_key,
        finality_provider_pub_key,
        staking_time,
    })
}

pub fn try_get_bbn_op_return_ouput(outputs: &[TxOut]) -> Option<BBNV0OpReturnOutput> {
    let mut result: Option<BBNV0OpReturnOutput> = None;
    for (vout, output) in outputs.iter().enumerate() {
        if output.script_pubkey.is_op_return() {
            let data = parse_bbn_op_return_data(&output.script_pubkey).ok();
            if let Some(data) = data {
                // this case should not happen as standard bitcoin node propagation rules
                // disallow multiple op return outputs in a single transaction. However, miner could
                // include multiple op return outputs in a single transaction. In such case, we should
                // skip the transaction.
                if result.is_some() {
                    return None;
                }
                result = Some(BBNV0OpReturnOutput {
                    op_return_output_idx: vout as u32,
                    op_return_data: data,
                });
            }
        }
    }
    result
}

fn try_get_bbn_staking_output(
    outputs: &[TxOut],
    staking_output_pk_script: &Script,
) -> Option<(TxOut, u32)> {
    let mut result: Option<(TxOut, u32)> = None;
    for (vout, output) in outputs.iter().enumerate() {
        if output.script_pubkey == *staking_output_pk_script {
            if result.is_some() {
                // bbn only allow for one staking output per transaction
                return None;
            }
            result = Some((output.clone(), vout as u32));
        }
    }
    result
}

fn assemble_taproot_script_tree(
    time_lock_script: ScriptBuf,
    unbonding_path_script: ScriptBuf,
    slashing_path_script: ScriptBuf,
) -> Result<TaprootBuilder, TaprootBuilderError> {
    let mut builder = TaprootBuilder::new();
    //We manually construct the tree here to ensure the order of the scripts is same as the babylon chain

    //TODO implement a `func AssembleTaprootScriptTree(leaves ...TapLeaf) *IndexedTapScriptTree` in the TaprootBuilder same as the go version
    builder = builder.add_leaf(2, time_lock_script)?;
    builder = builder.add_leaf(2, unbonding_path_script)?;
    builder = builder.add_leaf(1, slashing_path_script)?;

    Ok(builder)
}

#[cfg(test)]
mod tests {

    use super::*;
    use bitcoin::consensus::deserialize;

    #[test]
    fn test_sort_keys() {
        let sorted_keys = sort_keys(BBN_GLOBAL_PARAM_BBN1.get_covenant_pks());
        let sorted_keys_str = sorted_keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        //println!("sorted_keys: {:?}", sorted_keys_str);
        //the expected sorted keys are from the babylon chain go test
        let expected_sorted_keys_str = vec![
            "23b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
            "4b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
            "8242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
            "cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
            "d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
            "d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
            "de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c",
            "e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
            "f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
        ];
        assert_eq!(sorted_keys_str, expected_sorted_keys_str);
    }

    #[test]
    fn test_build_time_lock_script() {
        //Ensure the time lock script is the same as the babylon chain
        //https://github.com/babylonlabs-io/babylon/blob/07e4437f993d32a2f24d33531cf7de7ac87fd0d9/btcstaking/scripts_utils.go#L105
        let staker_pk = XOnlyPublicKey::from_str(
            "0b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393",
        )
        .unwrap();
        let script = build_time_lock_script(&staker_pk, 64000);
        let expected_script_hex =
            "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad0300fa00b2";
        assert_eq!(script.to_hex_string(), expected_script_hex);
    }

    #[test]
    fn test_covenant_multisig_script() {
        let keys = BBN_GLOBAL_PARAM_BBN1.get_covenant_pks();
        let script =
            build_multi_sig_script(&keys, BBN_GLOBAL_PARAM_BBN1.covenant_quorum, false).unwrap();
        //println!("script: {:?}", script.to_hex_string());
        let expected_script_hex = "2023b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1ac204b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9faba208242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7ba20cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204ba20d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967aeba20d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaaba20de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8cba20e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41cba20f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0ba569c";
        assert_eq!(script.to_hex_string(), expected_script_hex);
    }

    #[test]
    fn test_fp_multisig_script() {
        let fp_pub_key = XOnlyPublicKey::from_str(
            "db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923",
        )
        .unwrap();
        let script = build_multi_sig_script(&[fp_pub_key], 1, true).unwrap();
        let expected_script_hex =
            "20db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923ad";
        assert_eq!(script.to_hex_string(), expected_script_hex);
    }

    #[test]
    fn test_staker_sig_script() {
        let staker_pk = XOnlyPublicKey::from_str(
            "0b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393",
        )
        .unwrap();
        let script = build_single_key_sig_script(&staker_pk, true);
        let expected_script_hex =
            "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad";
        assert_eq!(script.to_hex_string(), expected_script_hex);
    }

    #[test]
    fn test_tapleaf_hash() {
        let script = ScriptBuf::from_hex(
            "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad0300fa00b2",
        )
        .unwrap();

        let leaf_hash = TapLeafHash::from_script(&script, LeafVersion::TapScript);
        //println!("leaf_hash: {:?}", leaf_hash);
        let expected_leaf_hash = TapLeafHash::from_str(
            "8a5d0707718a8f13727dd0c2f6976cf7c8b929ce9cf44976c8cb823d51128d37",
        )
        .unwrap();
        assert_eq!(leaf_hash, expected_leaf_hash);
    }

    #[test]
    fn test_taproot_script_holder() {
        let covenant_multisig_script_hex = "2023b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1ac204b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9faba208242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7ba20cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204ba20d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967aeba20d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaaba20de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8cba20e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41cba20f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0ba569c";
        let fp_multisig_script_hex =
            "20db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923ad";
        let timelock_script_hex =
            "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad0300fa00b2";
        let staker_sig_script_hex =
            "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad";

        let timelock_script = ScriptBuf::from_hex(timelock_script_hex).unwrap();
        let staker_sig_script = ScriptBuf::from_hex(staker_sig_script_hex).unwrap();
        let covenant_multisig_script = ScriptBuf::from_hex(covenant_multisig_script_hex).unwrap();
        let fp_multisig_script = ScriptBuf::from_hex(fp_multisig_script_hex).unwrap();

        let unbonding_path_script = aggregate_scripts(vec![
            staker_sig_script.clone(),
            covenant_multisig_script.clone(),
        ]);

        let slashing_path_script = aggregate_scripts(vec![
            staker_sig_script,
            fp_multisig_script,
            covenant_multisig_script,
        ]);

        let unspendable_key_path_key = *UNSPENDABLE_KEY_PATH_KEY;
        let holder = TaprootScriptHolder::new(
            &Secp256k1::new(),
            unspendable_key_path_key,
            timelock_script,
            unbonding_path_script,
            slashing_path_script,
        )
        .unwrap();
        let taproot_pk_script = holder.taproot_pk_script();
        // println!(
        //     "Taproot merkle root: {}",
        //     holder.script_tree.merkle_root().unwrap()
        // );
        // println!("Taproot PK script: {}", taproot_pk_script.to_hex_string());
        let expected_script_hex =
            "512082f93ece9366a9e680d152dbc0c487e181accbba145b5b72d00c820545064d44";
        assert_eq!(taproot_pk_script.to_hex_string(), expected_script_hex);
    }

    #[test]
    fn test_parse_bbn_op_return_data() {
        //https://mempool.space/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003
        let script = ScriptBuf::from_hex("6a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa00").unwrap();
        let bbn_op_return_data = parse_bbn_op_return_data(&script).unwrap();
        assert_eq!(bbn_op_return_data.tag, "bbn1".as_bytes().to_vec());
        assert_eq!(bbn_op_return_data.staking_time, 64000);
    }

    #[test]
    fn test_parse_tx() {
        //https://mempool.space/api/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003/hex
        let tx: Transaction = deserialize(&hex::decode("02000000000101113b905156fc89867c6d48cc3e5c3dc62558405325090ae0c141ccc57fafb5670000000000000000000380c629fc0600000022512082f93ece9366a9e680d152dbc0c487e181accbba145b5b72d00c820545064d440000000000000000496a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa0090e20000000000002251200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393014003d8ecbcb6f6eb069cd8ebd831606a80b43dbd1e4b78a95b09ceb2580dc61d0e2537770ef3e620310c7967d7db50895b356eba53c74e2dfddb3955b9ace8538515320d00").unwrap()).unwrap();
        let params = BBN_GLOBAL_PARAM_BBN1.clone();
        let expected_tag = &params.tag;
        let covenant_keys = params.get_covenant_pks();
        let covenant_quorum = params.covenant_quorum;
        let parsed_staking_tx =
            BBNParsedV0StakingTx::parse_from_tx(&tx, expected_tag, &covenant_keys, covenant_quorum)
                .unwrap();

        assert_eq!(parsed_staking_tx.op_return_output_idx, 1);

        assert_eq!(parsed_staking_tx.staking_output_idx, 0);
        assert_eq!(
            parsed_staking_tx.staking_output.value,
            Amount::from_sat(30000400000)
        );
    }

    //https://github.com/babylonlabs-io/staking-indexer/issues/26
    #[test]
    fn test_parse_tx2() {
        //https://mempool.space/tx/2aeeddb97b138ea622d9194818fa2fa3d8432125032ac1aec32461ae91d80b78
        let tx: Transaction = deserialize(&hex::decode("02000000000101a9f4558e50f0dcac3a624e806f38822bb5b83b02de946c2c2d5dac9b07843b99000000000046ffffff0284344c0000000000225120db09240cf52111e39179e50f5cf6c910a5478659ef94c29d668ebb9f469475450000000000000000496a4762626e3100d3d09d91bab234a9d21bf3c98092e82aec525caf67566edace08918408819689b3a838cbf2e61f2ecadf9f5924710e66dcf8212545884853073fe62c5ff5b949fa00014061a5eada8df8f5f3ce00f4389682530d268917f71631ad412a3d7c651342739033a868d4be039bb4dd0bc7c7ef5dada8c6a47171b07532a416fe2478be34303100000000").unwrap()).unwrap();
        let params = BBN_GLOBAL_PARAM_BBN1.clone();
        let expected_tag = &params.tag;
        let covenant_keys = params.get_covenant_pks();
        let covenant_quorum = params.covenant_quorum;
        let _parsed_staking_tx =
            BBNParsedV0StakingTx::parse_from_tx(&tx, expected_tag, &covenant_keys, covenant_quorum)
                .unwrap();
        // println!("tx lock time: {}", tx.lock_time);
        // println!("parsed_staking_tx: {:?}", parsed_staking_tx);
    }
}
