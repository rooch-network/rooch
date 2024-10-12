// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types;
use crate::{addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};
use anyhow::{anyhow, Result};
use bitcoin::{
    key::Secp256k1,
    opcodes::all::*,
    script::Builder,
    taproot::{LeafVersion, TapLeaf, TaprootBuilder, TaprootBuilderError, TaprootSpendInfo},
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
    moveos_std::tx_context::TxContext,
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub const MODULE_NAME: &IdentStr = ident_str!("bbn");

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
pub static BBN_GLOBAL_PARAM_BBN1: Lazy<BBNGlobalParam> = Lazy::new(|| BBNGlobalParam {
    version: 1,
    activation_height: 864790,
    staking_cap: 0,
    cap_height: 864799,
    tag: "62626e31".as_bytes().to_vec(),
    covenant_pks: vec![
        hex::decode("03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa").unwrap(),
        hex::decode("034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa").unwrap(),
        hex::decode("0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1").unwrap(),
        hex::decode("02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae").unwrap(),
        hex::decode("038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7").unwrap(),
        hex::decode("03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c").unwrap(),
        hex::decode("03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204").unwrap(),
        hex::decode("03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0").unwrap(),
        hex::decode("03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c").unwrap(),
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
pub struct BBNGlobalParam {
    pub version: u64,
    pub activation_height: u64,
    pub staking_cap: u64,
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

impl MoveStructType for BBNGlobalParam {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNGlobalParam");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNGlobalParam {
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

impl BBNGlobalParam {
    pub fn get_covenant_pks(&self) -> Vec<XOnlyPublicKey> {
        self.covenant_pks
            .iter()
            .map(|pk| XOnlyPublicKey::from(PublicKey::from_slice(pk).unwrap()))
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
    pub vout: u32,
    pub tag: Vec<u8>,
    pub staker_pub_key: Vec<u8>,
    pub finality_provider_pub_key: Vec<u8>,
    /// The stake time in block count
    pub staking_time: u16,
    /// The stake amount in satoshi
    pub staking_amount: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNOpReturnData {
    pub tag: Vec<u8>,
    pub staker_pub_key: Vec<u8>,
    pub finality_provider_pub_key: Vec<u8>,
    pub staking_time: u16,
}

impl MoveStructType for BBNOpReturnData {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNOpReturnData");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNOpReturnData {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U16,
        ])
    }
}

impl BBNOpReturnData {
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
pub struct BBNOpReturnOutput {
    pub vout: u32,
    pub op_return_data: BBNOpReturnData,
}

/// Rust bindings for BitcoinMove bitcoin module
pub struct BBNModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BBNModule<'a> {
    pub const TRY_GET_BBN_OP_RETURN_OUTPUT_FROM_TX_BYTES_FUNCTION_NAME: &'static IdentStr =
        ident_str!("try_get_bbn_op_return_output_from_tx_bytes");
    pub const IS_BBN_TX_FUNCTION_NAME: &'static IdentStr = ident_str!("is_bbn_tx");
    pub const PROCESS_BBN_TX_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("process_bbn_tx_entry");

    pub fn try_get_bbn_op_return_output_from_tx(
        &self,
        tx: Transaction,
    ) -> Result<Option<BBNOpReturnOutput>> {
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
                bcs::from_bytes::<MoveOption<BBNOpReturnOutput>>(&bbn_op_return_output.value)
                    .expect("should be a valid BBNOpReturnOutput")
            })
            .map_err(|e| anyhow::anyhow!("Failed to get bbn op return data: {:?}", e))?;
        Ok(bbn_op_return_output_opt.into())
    }

    pub fn is_bbn_tx(&self, txid: Txid) -> Result<bool> {
        let call = Self::create_function_call(
            Self::IS_BBN_TX_FUNCTION_NAME,
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

    pub fn create_process_bbn_tx_entry_call(&self, txid: Txid) -> Result<FunctionCall> {
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

//https://github.com/babylonlabs-io/babylon/blob/07e4437f993d32a2f24d33531cf7de7ac87fd0d9/btcstaking/types.go#L192

// const (
// 	// Point with unknown discrete logarithm defined in: https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
// 	// using it as internal public key effectively disables taproot key spends
// 	unspendableKeyPath = "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
// )

// var (
// 	unspendableKeyPathKey    = unspendableKeyPathInternalPubKeyInternal(unspendableKeyPath)
// )

/// Point with unknown discrete logarithm defined in: https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
/// using it as internal public key effectively disables taproot key spends
const UNSPENDABLE_KEY_PATH: &str =
    "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

static UNSPENDABLE_KEY_PATH_KEY: Lazy<XOnlyPublicKey> =
    Lazy::new(|| XOnlyPublicKey::from(PublicKey::from_str(UNSPENDABLE_KEY_PATH).unwrap()));

// type taprootScriptHolder struct {
// 	internalPubKey *btcec.PublicKey
// 	scriptTree     *txscript.IndexedTapScriptTree
// }

#[derive(Debug, Clone)]
struct TaprootScriptHolder {
    internal_pub_key: XOnlyPublicKey,
    script_tree: TaprootSpendInfo,
}

impl TaprootScriptHolder {
    pub fn new<C>(
        secp: &Secp256k1<C>,
        internal_pub_key: XOnlyPublicKey,
        scripts: Vec<ScriptBuf>,
    ) -> Result<Self>
    where
        C: bitcoin::secp256k1::Verification,
    {
        println!("Internal key: {:?}", internal_pub_key);

        if scripts.is_empty() {
            return Ok(Self {
                internal_pub_key,
                script_tree: TaprootSpendInfo::new_key_spend(secp, internal_pub_key, None),
            });
        }

        // Check for empty scripts
        if scripts.iter().any(|s| s.is_empty()) {
            return Err(anyhow!("Cannot build tree with empty script"));
        }

        // Check for duplicate scripts
        let mut seen_scripts = HashSet::new();
        for script in &scripts {
            if !seen_scripts.insert(script) {
                return Err(anyhow!("Duplicate script in provided scripts"));
            }
        }

        let (taproot_builder, _) = assemble_taproot_script_tree(scripts)?;
        let taproot_spend_info = taproot_builder
            .finalize(secp, internal_pub_key)
            .map_err(|e| anyhow!("Failed to finalize taproot tree: {:?}", e))?;

        println!(
            "Taproot merkle root: {:?}",
            taproot_spend_info.merkle_root()
        );
        println!("Output key: {:?}", taproot_spend_info.output_key());

        Ok(Self {
            internal_pub_key,
            script_tree: taproot_spend_info,
        })
    }

    fn taproot_pk_script(&self) -> ScriptBuf {
        let output_key = self.script_tree.output_key();
        let address = Address::p2tr_tweaked(output_key, Network::Bitcoin);
        address.script_pubkey()
    }
}

// type StakingInfo struct {
// 	StakingOutput         *wire.TxOut
// 	scriptHolder          *taprootScriptHolder
// 	timeLockPathLeafHash  chainhash.Hash
// 	unbondingPathLeafHash chainhash.Hash
// 	slashingPathLeafHash  chainhash.Hash
// }

#[derive(Debug, Clone)]
struct StakingInfo {
    staking_output: TxOut,
    script_holder: TaprootScriptHolder,
    time_lock_path_leaf_hash: TapLeafHash,
    unbonding_path_leaf_hash: TapLeafHash,
    slashing_path_leaf_hash: TapLeafHash,
}

// // BuildStakingInfo builds all Babylon specific BTC scripts that must
// // be committed to in the staking output.
// // Returned `StakingInfo` object exposes methods to build spend info for each
// // of the script spending paths which later must be included in the witness.
// // It is up to the caller to verify whether parameters provided to this function
// // obey parameters expected by Babylon chain.
// func BuildStakingInfo(
// 	stakerKey *btcec.PublicKey,
// 	fpKeys []*btcec.PublicKey,
// 	covenantKeys []*btcec.PublicKey,
// 	covenantQuorum uint32,
// 	stakingTime uint16,
// 	stakingAmount btcutil.Amount,
// 	net *chaincfg.Params,
// ) (*StakingInfo, error) {
// 	unspendableKeyPathKey := unspendableKeyPathInternalPubKey()

// 	babylonScripts, err := newBabylonScriptPaths(
// 		stakerKey,
// 		fpKeys,
// 		covenantKeys,
// 		covenantQuorum,
// 		stakingTime,
// 	)

// 	if err != nil {
// 		return nil, fmt.Errorf("%s: %w", errBuildingStakingInfo, err)
// 	}

// 	var unbondingPaths [][]byte
// 	unbondingPaths = append(unbondingPaths, babylonScripts.timeLockPathScript)
// 	unbondingPaths = append(unbondingPaths, babylonScripts.unbondingPathScript)
// 	unbondingPaths = append(unbondingPaths, babylonScripts.slashingPathScript)

// 	timeLockLeafHash := txscript.NewBaseTapLeaf(babylonScripts.timeLockPathScript).TapHash()
// 	unbondingPathLeafHash := txscript.NewBaseTapLeaf(babylonScripts.unbondingPathScript).TapHash()
// 	slashingLeafHash := txscript.NewBaseTapLeaf(babylonScripts.slashingPathScript).TapHash()

// 	sh, err := newTaprootScriptHolder(
// 		&unspendableKeyPathKey,
// 		unbondingPaths,
// 	)

// 	if err != nil {
// 		return nil, fmt.Errorf("%s: %w", errBuildingStakingInfo, err)
// 	}

// 	taprootPkScript, err := sh.taprootPkScript(net)

// 	if err != nil {
// 		return nil, fmt.Errorf("%s: %w", errBuildingStakingInfo, err)
// 	}

// 	stakingOutput := wire.NewTxOut(int64(stakingAmount), taprootPkScript)

// 	return &StakingInfo{
// 		StakingOutput:         stakingOutput,
// 		scriptHolder:          sh,
// 		timeLockPathLeafHash:  timeLockLeafHash,
// 		unbondingPathLeafHash: unbondingPathLeafHash,
// 		slashingPathLeafHash:  slashingLeafHash,
// 	}, nil
// }

// // babylonScriptPaths contains all possible babylon script paths
// // not every babylon output will contain all of those paths
// type babylonScriptPaths struct {
// 	// timeLockPathScript is the script path for normal unbonding
// 	// <Staker_PK> OP_CHECKSIGVERIFY  <Staking_Time_Blocks> OP_CHECKSEQUENCEVERIFY
// 	timeLockPathScript []byte
// 	// unbondingPathScript is the script path for on-demand early unbonding
// 	// <Staker_PK> OP_CHECKSIGVERIFY
// 	// <Covenant_PK1> OP_CHECKSIG ... <Covenant_PKN> OP_CHECKSIGADD M OP_NUMEQUAL
// 	unbondingPathScript []byte
// 	// slashingPathScript is the script path for slashing
// 	// <Staker_PK> OP_CHECKSIGVERIFY
// 	// <FP_PK1> OP_CHECKSIG ... <FP_PKN> OP_CHECKSIGADD 1 OP_NUMEQUALVERIFY
// 	// <Covenant_PK1> OP_CHECKSIG ... <Covenant_PKN> OP_CHECKSIGADD M OP_NUMEQUAL
// 	slashingPathScript []byte
// }

impl StakingInfo {
    fn build_staking_info(
        staker_key: &XOnlyPublicKey,
        fp_keys: &[XOnlyPublicKey],
        covenant_keys: &[XOnlyPublicKey],
        covenant_quorum: u32,
        staking_time: u16,
        staking_amount: u64,
        //net: &Params,
    ) -> Result<Self> {
        let unspendable_key_path_key = *UNSPENDABLE_KEY_PATH_KEY;
        let script_paths = BabylonScriptPaths::new(
            staker_key,
            fp_keys,
            covenant_keys,
            covenant_quorum,
            staking_time,
        )?;

        let taproot_script_holder = TaprootScriptHolder::new(
            &Secp256k1::new(),
            unspendable_key_path_key,
            vec![
                script_paths.time_lock_path_script.clone(),
                script_paths.unbonding_path_script.clone(),
                script_paths.slashing_path_script.clone(),
            ],
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
struct BabylonScriptPaths {
    time_lock_path_script: ScriptBuf,
    unbonding_path_script: ScriptBuf,
    slashing_path_script: ScriptBuf,
}

// func newBabylonScriptPaths(
// 	stakerKey *btcec.PublicKey,
// 	fpKeys []*btcec.PublicKey,
// 	covenantKeys []*btcec.PublicKey,
// 	covenantQuorum uint32,
// 	lockTime uint16,
// ) (*babylonScriptPaths, error) {
// 	if stakerKey == nil {
// 		return nil, fmt.Errorf("staker key is nil")
// 	}

// 	if err := checkForDuplicateKeys(stakerKey, fpKeys, covenantKeys); err != nil {
// 		return nil, fmt.Errorf("error building scripts: %w", err)
// 	}

// 	timeLockPathScript, err := buildTimeLockScript(stakerKey, lockTime)

// 	if err != nil {
// 		return nil, err
// 	}

// 	covenantMultisigScript, err := buildMultiSigScript(
// 		covenantKeys,
// 		covenantQuorum,
// 		// covenant multisig is always last in script so we do not run verify and leave
// 		// last value on the stack. If we do not leave at least one element on the stack
// 		// script will always error
// 		false,
// 	)

// 	if err != nil {
// 		return nil, err
// 	}

// 	stakerSigScript, err := buildSingleKeySigScript(stakerKey, true)

// 	if err != nil {
// 		return nil, err
// 	}

// 	fpMultisigScript, err := buildMultiSigScript(
// 		fpKeys,
// 		// we always require only one finality provider to sign
// 		1,
// 		// we need to run verify to clear the stack, as finality provider multisig is in the middle of the script
// 		true,
// 	)

// 	if err != nil {
// 		return nil, err
// 	}

// 	unbondingPathScript := aggregateScripts(
// 		stakerSigScript,
// 		covenantMultisigScript,
// 	)

// 	slashingPathScript := aggregateScripts(
// 		stakerSigScript,
// 		fpMultisigScript,
// 		covenantMultisigScript,
// 	)

// 	return &babylonScriptPaths{
// 		timeLockPathScript:  timeLockPathScript,
// 		unbondingPathScript: unbondingPathScript,
// 		slashingPathScript:  slashingPathScript,
// 	}, nil
// }
impl BabylonScriptPaths {
    fn new(
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

        Ok(BabylonScriptPaths {
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

// // buildMultiSigScript creates multisig script with given keys and signer threshold to
// // successfully execute script
// // it validates whether threshold is not greater than number of keys
// // If there is only one key provided it will return single key sig script
// // Note: It is up to the caller to ensure that the keys are unique
// func buildMultiSigScript(
// 	keys []*btcec.PublicKey,
// 	threshold uint32,
// 	withVerify bool,
// ) ([]byte, error) {
// 	if len(keys) == 0 {
// 		return nil, fmt.Errorf("no keys provided")
// 	}

// 	if int(threshold) > len(keys) {
// 		return nil, fmt.Errorf("required number of valid signers is greater than number of provided keys")
// 	}

// 	if len(keys) == 1 {
// 		// if we have only one key we can use single key sig script
// 		return buildSingleKeySigScript(keys[0], withVerify)
// 	}

// 	sortedKeys, err := prepareKeysForMultisigScript(keys)

// 	if err != nil {
// 		return nil, err
// 	}

// 	return assembleMultiSigScript(sortedKeys, threshold, withVerify)
// }

// // Only holder of private key for given pubKey can spend after relative lock time
// // SCRIPT: <StakerPk> OP_CHECKSIGVERIFY <Staking_Time_Blocks> OP_CHECKSEQUENCEVERIFY
// func buildTimeLockScript(
// 	pubKey *btcec.PublicKey,
// 	lockTime uint16,
// ) ([]byte, error) {
// 	builder := txscript.NewScriptBuilder()
// 	builder.AddData(schnorr.SerializePubKey(pubKey))
// 	builder.AddOp(txscript.OP_CHECKSIGVERIFY)
// 	builder.AddInt64(int64(lockTime))
// 	builder.AddOp(txscript.OP_CHECKSEQUENCEVERIFY)
// 	return builder.Script()
// }

// // Only holder of private key for given pubKey can spend
// // SCRIPT: <pubKey> OP_CHECKSIGVERIFY
// func buildSingleKeySigScript(
// 	pubKey *btcec.PublicKey,
// 	withVerify bool,
// ) ([]byte, error) {
// 	builder := txscript.NewScriptBuilder()
// 	builder.AddData(schnorr.SerializePubKey(pubKey))

// 	if withVerify {
// 		builder.AddOp(txscript.OP_CHECKSIGVERIFY)
// 	} else {
// 		builder.AddOp(txscript.OP_CHECKSIG)
// 	}
//     builder.script()
// }

// func aggregateScripts(scripts ...[]byte) []byte {
// 	if len(scripts) == 0 {
// 		return []byte{}
// 	}

// 	var finalScript []byte

// 	for _, script := range scripts {
// 		finalScript = append(finalScript, script...)
// 	}
// 	return finalScript
// }

// private helper to assemble multisig script
// if `withVerify` is true script will end with OP_NUMEQUALVERIFY otherwise with OP_NUMEQUAL
// SCRIPT: <Pk1> OP_CHEKCSIG <Pk2> OP_CHECKSIGADD <Pk3> OP_CHECKSIGADD ... <PkN> OP_CHECKSIGADD <threshold> OP_NUMEQUALVERIFY (or OP_NUMEQUAL)
// func assembleMultiSigScript(
// 	pubkeys []*btcec.PublicKey,
// 	threshold uint32,
// 	withVerify bool,
// ) ([]byte, error) {
// 	builder := txscript.NewScriptBuilder()

// 	for i, key := range pubkeys {
// 		builder.AddData(schnorr.SerializePubKey(key))
// 		if i == 0 {
// 			builder.AddOp(txscript.OP_CHECKSIG)
// 		} else {
// 			builder.AddOp(txscript.OP_CHECKSIGADD)
// 		}
// 	}

// 	builder.AddInt64(int64(threshold))
// 	if withVerify {
// 		builder.AddOp(txscript.OP_NUMEQUALVERIFY)
// 	} else {
// 		builder.AddOp(txscript.OP_NUMEQUAL)
// 	}

// 	return builder.Script()
// }

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

fn prepare_keys_for_multisig_script(keys: &[XOnlyPublicKey]) -> Vec<XOnlyPublicKey> {
    let mut keys = keys.to_vec();
    sort_keys(keys)
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
    let keys = prepare_keys_for_multisig_script(keys);
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

fn parse_bbn_op_return_data(script: &Script) -> Result<BBNOpReturnData> {
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

    Ok(BBNOpReturnData {
        tag,
        staker_pub_key,
        finality_provider_pub_key,
        staking_time,
    })
}

fn try_get_bbn_op_return_output(tx: &Transaction) -> Option<BBNOpReturnOutput> {
    let mut result: Option<BBNOpReturnOutput> = None;
    for (vout, output) in tx.output.iter().enumerate() {
        if output.script_pubkey.is_op_return() {
            let data = parse_bbn_op_return_data(&output.script_pubkey).ok();
            if let Some(data) = data {
                if result.is_some() {
                    return None;
                }
                result = Some(BBNOpReturnOutput {
                    vout: vout as u32,
                    op_return_data: data,
                });
            }
        }
    }
    result
}

fn try_get_bbn_staking_output(
    tx: &Transaction,
    staking_output_pk_script: &Script,
) -> Option<(TxOut, u32)> {
    let mut result: Option<(TxOut, u32)> = None;
    for (vout, output) in tx.output.iter().enumerate() {
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
    scripts: Vec<ScriptBuf>,
) -> Result<(TaprootBuilder, HashMap<TapLeafHash, usize>)> {
    let mut leaf_index = HashMap::new();

    println!("Number of scripts: {}", scripts.len());

    // Print each script
    for (i, script) in scripts.iter().enumerate() {
        println!("Script {}: {}", i, script.to_hex_string());
        let leaf_hash = TapLeafHash::from_script(script, LeafVersion::TapScript);
        println!("Leaf hash {}: {:?}", i, leaf_hash);
        leaf_index.insert(leaf_hash, i);
    }

    // Create script weights
    let script_weights: Vec<(u32, ScriptBuf)> = scripts
        .into_iter()
        .enumerate()
        .map(|(i, script)| {
            (1u32, script) // Using weight 1 for all scripts
        })
        .collect();

    // Use the with_huffman_tree method to build the tree
    let builder = TaprootBuilder::with_huffman_tree(script_weights)
        .map_err(|e| anyhow::anyhow!("Failed to build Taproot tree: {:?}", e))?;

    // Print the structure of the built tree
    print_tree_structure(&builder);

    Ok((builder, leaf_index))
}

fn print_tree_structure(builder: &TaprootBuilder) {
    let tree = builder.clone().try_into_taptree().unwrap();
    println!("Tree structure:");
    for leaf in tree.script_leaves() {
        println!(
            "Leaf: {:?}, merkle branch: {:?}",
            leaf.script().to_hex_string(),
            leaf.merkle_branch()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::consensus::{deserialize, Decodable};

    #[test]
    fn test_sort_keys() {
        let sorted_keys = sort_keys(BBN_GLOBAL_PARAM_BBN1.get_covenant_pks());
        let sorted_keys_str = sorted_keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        println!("sorted_keys: {:?}", sorted_keys_str);
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
        println!("script: {:?}", script.to_hex_string());
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
        println!("leaf_hash: {:?}", leaf_hash);
        let expected_leaf_hash = TapLeafHash::from_str(
            "378d12513d82cbc87649f49cce29b9c8f76c97f6c2d07d72138f8a7107075d8a",
        )
        .unwrap();
        assert_eq!(leaf_hash, expected_leaf_hash);
    }

    // func TestTaprootScriptHolder(t *testing.T) {
    // 	covenantMultisigScriptHex := "2023b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1ac204b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9faba208242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7ba20cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204ba20d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967aeba20d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaaba20de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8cba20e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41cba20f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0ba569c"
    // 	fpMulitsigScriptHex := "20db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923ad"
    // 	timelockScriptHex := "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad0300fa00b2"
    // 	stakerSigScriptHex := "200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad"

    // 	timelockScript, err := hex.DecodeString(timelockScriptHex)
    // 	require.NoError(t, err)
    // 	stakerSigScript, err := hex.DecodeString(stakerSigScriptHex)
    // 	require.NoError(t, err)
    // 	covenantMultisigScript, err := hex.DecodeString(covenantMultisigScriptHex)
    // 	require.NoError(t, err)
    // 	fpMultisigScript, err := hex.DecodeString(fpMulitsigScriptHex)
    // 	require.NoError(t, err)

    // 	unbondingPathScript := aggregateScripts(
    // 		stakerSigScript,
    // 		covenantMultisigScript,
    // 	)

    // 	slashingPathScript := aggregateScripts(
    // 		stakerSigScript,
    // 		fpMultisigScript,
    // 		covenantMultisigScript,
    // 	)

    // 	fmt.Println(hex.EncodeToString(unbondingPathScript))
    // 	fmt.Println(hex.EncodeToString(slashingPathScript))

    // 	var unbondingPaths [][]byte
    // 	unbondingPaths = append(unbondingPaths, timelockScript)
    // 	unbondingPaths = append(unbondingPaths, unbondingPathScript)
    // 	unbondingPaths = append(unbondingPaths, slashingPathScript)

    // 	unspendableKeyPathKey := unspendableKeyPathInternalPubKey()
    // 	holder, err := newTaprootScriptHolder(&unspendableKeyPathKey, unbondingPaths)
    // 	require.NoError(t, err)
    // 	taprootPkScript, err := holder.taprootPkScript(&chaincfg.MainNetParams)
    // 	require.NoError(t, err)
    // 	fmt.Println(hex.EncodeToString(taprootPkScript))
    // }
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

        println!(
            "Unbonding path script: {}",
            unbonding_path_script.to_hex_string()
        );
        println!(
            "Slashing path script: {}",
            slashing_path_script.to_hex_string()
        );

        let unbonding_paths = vec![timelock_script, unbonding_path_script, slashing_path_script];

        let unspendable_key_path_key = *UNSPENDABLE_KEY_PATH_KEY;
        let holder =
            TaprootScriptHolder::new(&Secp256k1::new(), unspendable_key_path_key, unbonding_paths)
                .unwrap();
        let taproot_pk_script = holder.taproot_pk_script();
        println!(
            "Taproot merkle root: {}",
            holder.script_tree.merkle_root().unwrap()
        );
        println!("Taproot PK script: {}", taproot_pk_script.to_hex_string());
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
    fn test_build_stake_info() {
        //https://mempool.space/api/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003/hex
        let tx: Transaction = deserialize(&hex::decode("02000000000101113b905156fc89867c6d48cc3e5c3dc62558405325090ae0c141ccc57fafb5670000000000000000000380c629fc0600000022512082f93ece9366a9e680d152dbc0c487e181accbba145b5b72d00c820545064d440000000000000000496a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa0090e20000000000002251200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393014003d8ecbcb6f6eb069cd8ebd831606a80b43dbd1e4b78a95b09ceb2580dc61d0e2537770ef3e620310c7967d7db50895b356eba53c74e2dfddb3955b9ace8538515320d00").unwrap()).unwrap();
        let bbn_op_return_output = try_get_bbn_op_return_output(&tx);
        assert!(bbn_op_return_output.is_some());
        let bbn_op_return_data = bbn_op_return_output.unwrap().op_return_data;
        println!("bbn_op_return_data: {:?}", bbn_op_return_data);
        println!(
            "staker_pub_key: {:?}",
            bbn_op_return_data.staker_pub_key().unwrap().to_string()
        );
        println!(
            "finality_provider_pub_key: {:?}",
            bbn_op_return_data
                .finality_provider_pub_key()
                .unwrap()
                .to_string()
        );
        let params = BBN_GLOBAL_PARAM_BBN1.clone();
        let stake_info = StakingInfo::build_staking_info(
            &bbn_op_return_data.staker_pub_key().unwrap(),
            &[bbn_op_return_data.finality_provider_pub_key().unwrap()],
            &params.get_covenant_pks(),
            params.covenant_quorum,
            bbn_op_return_data.staking_time,
            0,
        )
        .unwrap();
        println!("stake_info: {:?}", stake_info);
        let staking_output_opt =
            try_get_bbn_staking_output(&tx, &stake_info.staking_output.script_pubkey);
        assert!(staking_output_opt.is_some());
        let (staking_output, staking_output_index) = staking_output_opt.unwrap();
        assert_eq!(staking_output_index, 0);
        assert_eq!(staking_output.value, Amount::from_sat(30000400000));
    }
}
