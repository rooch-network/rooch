// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::jsonrpc_types::StrView;
use bitcoin::ScriptBuf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    address::{AddressTypeView, AddressView},
    amount::AmountView,
    hash_types::{BlockHashView, TxidView},
    network::{NetworkCheckedView, NetworkUncheckedView},
};

pub type X160View = StrView<bitcoin::bip32::XKeyIdentifier>;

impl FromStr for X160View {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(bitcoin::bip32::XKeyIdentifier::from_str(s)?))
    }
}

impl From<X160View> for bitcoin::bip32::XKeyIdentifier {
    fn from(value: X160View) -> Self {
        value.0
    }
}

impl std::fmt::Display for X160View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub type ScriptBufView = StrView<ScriptBuf>;

impl FromStr for ScriptBufView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ScriptBuf::from_hex(s)?))
    }
}

impl From<ScriptBufView> for ScriptBuf {
    fn from(value: ScriptBufView) -> Self {
        value.0
    }
}

impl std::fmt::Display for ScriptBufView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bip9SoftforkStatusView {
    Defined,
    Started,
    LockedIn,
    Active,
    Failed,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Bip9SoftforkStatisticsView {
    pub period: u32,
    pub threshold: Option<u32>,
    pub elapsed: u32,
    pub count: u32,
    pub possible: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Bip9SoftforkInfoView {
    pub status: Bip9SoftforkStatusView,
    pub bit: Option<u8>,
    // Can be -1 for 0.18.x inactive ones.
    pub start_time: StrView<i64>,
    pub timeout: StrView<u64>,
    pub since: u32,
    pub statistics: Option<Bip9SoftforkStatisticsView>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SoftforkTypeView {
    Buried,
    Bip9,
    #[serde(other)]
    Other,
}

/// Status of a softfork
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SoftforkView {
    #[serde(rename = "type")]
    pub type_: SoftforkTypeView,
    pub bip9: Option<Bip9SoftforkInfoView>,
    pub height: Option<u32>,
    pub active: bool,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Serialize, JsonSchema)]
pub struct GetBalancesResultEntryView {
    pub trusted: AmountView,
    pub untrusted_pending: AmountView,
    pub immature: AmountView,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBalancesResultView {
    pub mine: GetBalancesResultEntryView,
    pub watchonly: Option<GetBalancesResultEntryView>,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScriptPubkeyTypeView {
    Nonstandard,
    Pubkey,
    PubkeyHash,
    ScriptHash,
    MultiSig,
    NullData,
    Witness_v0_KeyHash,
    Witness_v0_ScriptHash,
    Witness_v1_Taproot,
    Witness_Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResultVoutScriptPubKeyView {
    pub asm: String,
    pub hex: StrView<Vec<u8>>,
    pub req_sigs: Option<StrView<usize>>,
    #[serde(rename = "type")]
    pub type_: Option<ScriptPubkeyTypeView>,
    // Deprecated in Bitcoin Core 22
    #[serde(default)]
    pub addresses: Vec<AddressView<NetworkCheckedView>>,
    // Added in Bitcoin Core 22
    #[serde(default)]
    pub address: Option<AddressView<NetworkUncheckedView>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetTxOutResultView {
    pub bestblock: BlockHashView,
    pub confirmations: u32,
    pub value: AmountView,
    pub script_pub_key: GetRawTransactionResultVoutScriptPubKeyView,
    pub coinbase: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetWalletInfoResultView {
    #[serde(rename = "walletname")]
    pub wallet_name: String,
    #[serde(rename = "walletversion")]
    pub wallet_version: u32,
    pub balance: AmountView,
    pub unconfirmed_balance: AmountView,
    pub immature_balance: AmountView,
    #[serde(rename = "txcount")]
    pub tx_count: StrView<usize>,
    #[serde(rename = "keypoololdest")]
    pub keypool_oldest: Option<StrView<usize>>,
    #[serde(rename = "keypoolsize")]
    pub keypool_size: StrView<usize>,
    #[serde(rename = "keypoolsize_hd_internal")]
    pub keypool_size_hd_internal: StrView<usize>,
    pub unlocked_until: Option<StrView<u64>>,
    #[serde(rename = "paytxfee")]
    pub pay_tx_fee: AmountView,
    #[serde(rename = "hdseedid")]
    pub hd_seed_id: Option<X160View>,
    pub private_keys_enabled: bool,
    pub avoid_reuse: Option<bool>,
    pub scanning: Option<ScanningDetailsView>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ScanningDetailsView {
    Scanning {
        duration: StrView<usize>,
        progress: f32,
    },
    /// The bool in this field will always be false.
    NotScanning(bool),
}

// Used for createrawtransaction argument.
#[derive(Serialize, Clone, PartialEq, Eq, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRawTransactionInputView {
    pub txid: TxidView,
    pub vout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoadWalletResultView {
    pub name: String,
    pub warning: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionOptionsView {
    /// For a transaction with existing inputs, automatically include more if they are not enough (default true).
    /// Added in Bitcoin Core v0.21
    #[serde(rename = "add_inputs", skip_serializing_if = "Option::is_none")]
    pub add_inputs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_address: Option<AddressView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_position: Option<u32>,
    #[serde(rename = "change_type", skip_serializing_if = "Option::is_none")]
    pub change_type: Option<AddressTypeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_watching: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_unspents: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_rate: Option<AmountView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtract_fee_from_outputs: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaceable: Option<bool>,
    #[serde(rename = "conf_target", skip_serializing_if = "Option::is_none")]
    pub conf_target: Option<u32>,
    #[serde(rename = "estimate_mode", skip_serializing_if = "Option::is_none")]
    pub estimate_mode: Option<EstimateModeView>,
}

// Custom types for input arguments.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum EstimateModeView {
    Unset,
    Economical,
    Conservative,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionResultView {
    pub hex: StrView<Vec<u8>>,
    pub fee: AmountView,
    #[serde(rename = "changepos")]
    pub change_position: i32,
}

// Used for signrawtransaction argument.
#[derive(Serialize, Clone, PartialEq, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignRawTransactionInputView {
    pub txid: TxidView,
    pub vout: u32,
    pub script_pub_key: ScriptBufView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeem_script: Option<ScriptBufView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<AmountView>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListUnspentResultEntryView {
    pub txid: TxidView,
    pub vout: u32,
    pub address: Option<AddressView<NetworkUncheckedView>>,
    pub label: Option<String>,
    pub redeem_script: Option<ScriptBufView>,
    pub witness_script: Option<ScriptBufView>,
    pub script_pub_key: ScriptBufView,
    pub amount: AmountView,
    pub confirmations: u32,
    pub spendable: bool,
    pub solvable: bool,
    #[serde(rename = "desc")]
    pub descriptor: Option<String>,
    pub safe: bool,
}

/// Outpoint that serializes and deserializes as a map, instead of a string,
/// for use as RPC arguments
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct JsonOutPointView {
    pub txid: TxidView,
    pub vout: u32,
}

/// Used to represent an address type.
#[derive(Copy, Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RpcAddressTypeView {
    Legacy,
    P2shSegwit,
    Bech32,
    Bech32m,
}

/// Models the result of "getdescriptorinfo"
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetDescriptorInfoResultView {
    pub descriptor: String,
    pub checksum: Option<String>,
    #[serde(rename = "isrange")]
    pub is_range: bool,
    #[serde(rename = "issolvable")]
    pub is_solvable: bool,
    #[serde(rename = "hasprivatekeys")]
    pub has_private_keys: bool,
}

/// A import request for importdescriptors.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ImportDescriptorsView {
    #[serde(rename = "desc")]
    pub descriptor: String,
    pub timestamp: TimestampView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(StrView<usize>, StrView<usize>)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_index: Option<StrView<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, Deserialize, Serialize, JsonSchema)]
pub enum TimestampView {
    Now,
    Time(StrView<u64>),
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ImportMultiResultView {
    pub success: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub error: Option<ImportMultiResultErrorView>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ImportMultiResultErrorView {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListTransactionResultView {
    #[serde(flatten)]
    pub info: WalletTxInfoView,
    #[serde(flatten)]
    pub detail: GetTransactionResultDetailView,

    pub trusted: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct WalletTxInfoView {
    pub confirmations: i32,
    pub blockhash: Option<BlockHashView>,
    pub blockindex: Option<StrView<usize>>,
    pub blocktime: Option<StrView<u64>>,
    pub blockheight: Option<u32>,
    pub txid: TxidView,
    pub time: StrView<u64>,
    pub timereceived: StrView<u64>,
    #[serde(rename = "bip125-replaceable")]
    pub bip125_replaceable: Bip125ReplaceableView,
    /// Conflicting transaction ids
    #[serde(rename = "walletconflicts")]
    pub wallet_conflicts: Vec<TxidView>,
}

/// Enum to represent the BIP125 replaceable status for a transaction.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Bip125ReplaceableView {
    Yes,
    No,
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetTransactionResultDetailView {
    pub address: Option<AddressView<NetworkUncheckedView>>,
    pub category: GetTransactionResultDetailCategoryView,
    pub amount: SignedAmountView,
    pub label: Option<String>,
    pub vout: u32,
    pub fee: Option<SignedAmountView>,
    pub abandoned: Option<bool>,
}

/// Enum to represent the category of a transaction.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GetTransactionResultDetailCategoryView {
    Send,
    Receive,
    Generate,
    Immature,
    Orphan,
}

/// SignedAmount
///
/// The [SignedAmount] type can be used to express Bitcoin amounts that support
/// arithmetic and conversion to various denominations.
///
///
/// Warning!
///
/// This type implements several arithmetic operations from [core::ops].
/// To prevent errors due to overflow or underflow when using these operations,
/// it is advised to instead use the checked arithmetic methods whose names
/// start with `checked_`.  The operations from [core::ops] that [Amount]
/// implements will panic when overflow or underflow occurs.
///
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "serde", serde(with = "bitcoin::amount::serde::as_btc"))]
pub struct SignedAmountView(i64);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescriptorView {
    pub desc: String,
    pub timestamp: TimestampView,
    pub active: bool,
    pub internal: Option<bool>,
    pub range: Option<(StrView<u64>, StrView<u64>)>,
    pub next: Option<StrView<u64>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListDescriptorsResultView {
    pub wallet_name: String,
    pub descriptors: Vec<DescriptorView>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListWalletDirResultView {
    pub wallets: Vec<ListWalletDirItemView>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListWalletDirItemView {
    pub name: String,
}

/// Models the result of "walletprocesspsbt"
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct WalletProcessPsbtResultView {
    pub psbt: String,
    pub complete: bool,
}

/// Models the result of "finalizepsbt"
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct FinalizePsbtResultView {
    pub psbt: Option<String>,
    pub hex: Option<StrView<Vec<u8>>>,
    pub complete: bool,
}
