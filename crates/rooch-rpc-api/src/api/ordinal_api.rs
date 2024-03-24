// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;
use serde_json::Value;
use std::collections::HashMap;

use crate::jsonrpc_types::btc::transaction::TxidView;
use crate::jsonrpc_types::ordinal::address::AddressView;
use crate::jsonrpc_types::ordinal::bitcoin::GetBlockchainInfoResultView;
use crate::jsonrpc_types::ordinal::hash_types::BlockHashView;
use crate::jsonrpc_types::ordinal::network::{GetNetworkInfoResultView, NetworkUncheckedView};
use crate::jsonrpc_types::ordinal::rpc::{
    CreateRawTransactionInputView, EstimateModeView, FinalizePsbtResultView,
    FundRawTransactionOptionsView, FundRawTransactionResultView, GetBalancesResultView,
    GetDescriptorInfoResultView, GetTxOutResultView, GetWalletInfoResultView,
    ImportDescriptorsView, ImportMultiResultView, JsonOutPointView, ListDescriptorsResultView,
    ListTransactionResultView, ListUnspentResultEntryView, ListWalletDirResultView,
    LoadWalletResultView, RpcAddressTypeView, SignRawTransactionInputView,
    WalletProcessPsbtResultView,
};
use crate::jsonrpc_types::StrView;

// TODO: error: this function has too many arguments (12/10)
#[open_rpc(namespace = "ordinal")]
#[rpc(server, client, namespace = "ordinal")]
#[async_trait]
pub trait OrdinalAPI {
    #[method(name = "getblockchaininfo")]
    fn get_blockchain_info(&self) -> RpcResult<GetBlockchainInfoResultView>;

    #[method(name = "getnetworkinfo")]
    fn get_network_info(&self) -> RpcResult<GetNetworkInfoResultView>;

    #[method(name = "getbalances")]
    fn get_balances(&self) -> RpcResult<GetBalancesResultView>;

    #[method(name = "getbestblockhash")]
    fn get_best_block_hash(&self) -> RpcResult<BlockHashView>;

    #[method(name = "getblockhash")]
    fn get_block_hash(&self, height: StrView<usize>) -> RpcResult<BlockHashView>;

    #[method(name = "getblockheader")]
    fn get_block_header(&self, block_hash: BlockHashView, verbose: bool) -> RpcResult<Value>;

    #[method(name = "getblock")]
    fn get_block(&self, blockhash: BlockHashView, verbosity: StrView<u64>) -> RpcResult<String>;

    #[method(name = "getblockcount")]
    fn get_block_count(&self) -> RpcResult<StrView<u64>>;

    #[method(name = "gettxout")]
    fn get_tx_out(
        &self,
        txid: TxidView,
        vout: u32,
        include_mempool: Option<bool>,
    ) -> RpcResult<Option<GetTxOutResultView>>;

    #[method(name = "getwalletinfo")]
    fn get_wallet_info(&self) -> RpcResult<GetWalletInfoResultView>;

    #[method(name = "createrawtransaction")]
    fn create_raw_transaction(
        &self,
        utxos: Vec<CreateRawTransactionInputView>,
        outs: HashMap<String, f64>,
        locktime: Option<StrView<i64>>,
        replaceable: Option<bool>,
    ) -> RpcResult<String>;

    #[method(name = "createwallet")]
    fn create_wallet(
        &self,
        name: String,
        disable_private_keys: Option<bool>,
        blank: Option<bool>,
        passphrase: Option<String>,
        avoid_reuse: Option<bool>,
    ) -> RpcResult<LoadWalletResultView>;

    #[method(name = "fundrawtransaction")]
    fn fund_raw_transaction(
        &self,
        tx: String,
        options: Option<FundRawTransactionOptionsView>,
        is_witness: Option<bool>,
    ) -> RpcResult<FundRawTransactionResultView>;

    #[method(name = "signrawtransactionwithwallet")]
    fn sign_raw_transaction_with_wallet(
        &self,
        tx: String,
        utxos: Option<Vec<SignRawTransactionInputView>>,
        sighash_type: Option<()>,
    ) -> RpcResult<Value>;

    #[method(name = "sendrawtransaction")]
    fn send_raw_transaction(&self, tx: String) -> RpcResult<String>;

    // TODO: error: this function has too many arguments (12/10)
    #[method(name = "sendtoaddress")]
    fn send_to_address(
        &self,
        address: AddressView<NetworkUncheckedView>,
        amount: f64,
        comment: Option<String>,
        comment_to: Option<String>,
        subtract_fee: Option<bool>,
        replaceable: Option<bool>,
        confirmation_target: Option<u32>,
        estimate_mode: Option<EstimateModeView>,
        avoid_reuse: Option<bool>,
        fee_rate: Option<f64>,
        verbose: Option<bool>,
    ) -> RpcResult<TxidView>;

    #[method(name = "gettransaction")]
    fn get_transaction(&self, txid: TxidView, include_watchonly: Option<bool>) -> RpcResult<Value>;

    #[method(name = "getrawtransaction")]
    fn get_raw_transaction(
        &self,
        txid: TxidView,
        verbose: Option<bool>,
        blockhash: Option<BlockHashView>,
    ) -> RpcResult<Value>;

    #[method(name = "listunspent")]
    fn list_unspent(
        &self,
        minconf: Option<StrView<usize>>,
        maxconf: Option<StrView<usize>>,
        address: Option<AddressView<NetworkUncheckedView>>,
        include_unsafe: Option<bool>,
        query_options: Option<String>,
    ) -> RpcResult<Vec<ListUnspentResultEntryView>>;

    #[method(name = "listlockunspent")]
    fn list_lock_unspent(&self) -> RpcResult<Vec<JsonOutPointView>>;

    #[method(name = "getrawchangeaddress")]
    fn get_raw_change_address(
        &self,
        address_type: Option<RpcAddressTypeView>,
    ) -> RpcResult<AddressView>;

    #[method(name = "getdescriptorinfo")]
    fn get_descriptor_info(&self, desc: String) -> RpcResult<GetDescriptorInfoResultView>;

    #[method(name = "importdescriptors")]
    fn import_descriptors(
        &self,
        req: Vec<ImportDescriptorsView>,
    ) -> RpcResult<Vec<ImportMultiResultView>>;

    #[method(name = "getnewaddress")]
    fn get_new_address(
        &self,
        label: Option<String>,
        address_type: Option<RpcAddressTypeView>,
    ) -> RpcResult<AddressView>;

    #[method(name = "listtransactions")]
    fn list_transactions(
        &self,
        label: Option<String>,
        count: Option<u16>,
        skip: Option<StrView<usize>>,
        include_watchonly: Option<bool>,
    ) -> RpcResult<Vec<ListTransactionResultView>>;

    #[method(name = "lockunspent")]
    fn lock_unspent(&self, unlock: bool, outputs: Vec<JsonOutPointView>) -> RpcResult<bool>;

    #[method(name = "listdescriptors")]
    fn list_descriptors(
        &self,
        _with_private_keys: Option<bool>,
    ) -> RpcResult<ListDescriptorsResultView>;

    #[method(name = "loadwallet")]
    fn load_wallet(&self, wallet: String) -> RpcResult<LoadWalletResultView>;

    #[method(name = "listwallets")]
    fn list_wallets(&self) -> RpcResult<Vec<String>>;

    #[method(name = "listwalletdir")]
    fn list_wallet_dir(&self) -> RpcResult<ListWalletDirResultView>;

    #[method(name = "walletprocesspsbt")]
    fn wallet_process_psbt(
        &self,
        psbt: String,
        sign: Option<bool>,
        sighash_type: Option<()>,
        bip32derivs: Option<bool>,
    ) -> RpcResult<WalletProcessPsbtResultView>;

    #[method(name = "finalizepsbt")]
    fn finalize_psbt(
        &self,
        psbt: String,
        extract: Option<bool>,
    ) -> RpcResult<FinalizePsbtResultView>;
}
