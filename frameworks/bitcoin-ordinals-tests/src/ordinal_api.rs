// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{FundRawTransactionOptions, FundRawTransactionResult, HashMap};
use bitcoin::{address::NetworkUnchecked, Address, BlockHash, Txid};
use bitcoincore_rpc::json::{
    CreateRawTransactionInput, EstimateMode, FinalizePsbtResult, GetBalancesResult,
    GetBlockchainInfoResult, GetDescriptorInfoResult, GetNetworkInfoResult, GetTxOutResult,
    GetWalletInfoResult, ImportDescriptors, ImportMultiResult, ListDescriptorsResult,
    ListTransactionResult, ListUnspentResultEntry, ListWalletDirResult, LoadWalletResult,
    SignRawTransactionInput, WalletProcessPsbtResult,
};
use bitcoincore_rpc::JsonOutPoint;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;
use serde_json::Value;

#[open_rpc(namespace = "ordinal")]
#[rpc(server, client, namespace = "ordinal")]
#[async_trait]
pub trait OrdinalAPI {
    #[method(name = "getblockchaininfo")]
    fn get_blockchain_info(&self) -> RpcResult<GetBlockchainInfoResult>;

    #[method(name = "getnetworkinfo")]
    fn get_network_info(&self) -> RpcResult<GetNetworkInfoResult>;

    #[method(name = "getbalances")]
    fn get_balances(&self) -> RpcResult<GetBalancesResult>;

    #[method(name = "getblockhash")]
    fn get_block_hash(&self, height: usize) -> RpcResult<BlockHash>;

    #[method(name = "getblockheader")]
    fn get_block_header(&self, block_hash: BlockHash, verbose: bool) -> RpcResult<Value>;

    #[method(name = "getblock")]
    fn get_block(&self, blockhash: BlockHash, verbosity: u64) -> RpcResult<String>;

    #[method(name = "getblockcount")]
    fn get_block_count(&self) -> RpcResult<u64>;

    #[method(name = "gettxout")]
    fn get_tx_out(
        &self,
        txid: Txid,
        vout: u32,
        include_mempool: Option<bool>,
    ) -> RpcResult<Option<GetTxOutResult>>;

    #[method(name = "getwalletinfo")]
    fn get_wallet_info(&self) -> RpcResult<GetWalletInfoResult>;

    #[method(name = "createrawtransaction")]
    fn create_raw_transaction(
        &self,
        utxos: Vec<CreateRawTransactionInput>,
        outs: HashMap<String, f64>,
        locktime: Option<i64>,
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
    ) -> RpcResult<LoadWalletResult>;

    #[method(name = "fundrawtransaction")]
    fn fund_raw_transaction(
        &self,
        tx: String,
        options: Option<FundRawTransactionOptions>,
        is_witness: Option<bool>,
    ) -> RpcResult<FundRawTransactionResult>;

    #[method(name = "signrawtransactionwithwallet")]
    fn sign_raw_transaction_with_wallet(
        &self,
        tx: String,
        utxos: Option<Vec<SignRawTransactionInput>>,
        sighash_type: Option<()>,
    ) -> RpcResult<Value>;

    #[method(name = "sendrawtransaction")]
    fn send_raw_transaction(&self, tx: String) -> RpcResult<String>;

    #[method(name = "sendtoaddress")]
    fn send_to_address(
        &self,
        address: Address<NetworkUnchecked>,
        amount: f64,
        comment: Option<String>,
        comment_to: Option<String>,
        subtract_fee: Option<bool>,
        replaceable: Option<bool>,
        confirmation_target: Option<u32>,
        estimate_mode: Option<EstimateMode>,
        avoid_reuse: Option<bool>,
        fee_rate: Option<f64>,
        verbose: Option<bool>,
    ) -> RpcResult<Txid>;

    #[method(name = "gettransaction")]
    fn get_transaction(&self, txid: Txid, include_watchonly: Option<bool>) -> RpcResult<Value>;

    #[method(name = "getrawtransaction")]
    fn get_raw_transaction(
        &self,
        txid: Txid,
        verbose: Option<bool>,
        blockhash: Option<BlockHash>,
    ) -> RpcResult<Value>;

    #[method(name = "listunspent")]
    fn list_unspent(
        &self,
        minconf: Option<usize>,
        maxconf: Option<usize>,
        address: Option<Address<NetworkUnchecked>>,
        include_unsafe: Option<bool>,
        query_options: Option<String>,
    ) -> RpcResult<Vec<ListUnspentResultEntry>>;

    #[method(name = "listlockunspent")]
    fn list_lock_unspent(&self) -> RpcResult<Vec<JsonOutPoint>>;

    #[method(name = "getrawchangeaddress")]
    fn get_raw_change_address(
        &self,
        address_type: Option<bitcoincore_rpc::json::AddressType>,
    ) -> RpcResult<Address>;

    #[method(name = "getdescriptorinfo")]
    fn get_descriptor_info(&self, desc: String) -> RpcResult<GetDescriptorInfoResult>;

    #[method(name = "importdescriptors")]
    fn import_descriptors(&self, req: Vec<ImportDescriptors>) -> RpcResult<Vec<ImportMultiResult>>;

    #[method(name = "getnewaddress")]
    fn get_new_address(
        &self,
        label: Option<String>,
        address_type: Option<bitcoincore_rpc::json::AddressType>,
    ) -> RpcResult<Address>;

    #[method(name = "listtransactions")]
    fn list_transactions(
        &self,
        label: Option<String>,
        count: Option<u16>,
        skip: Option<usize>,
        include_watchonly: Option<bool>,
    ) -> RpcResult<Vec<ListTransactionResult>>;

    #[method(name = "lockunspent")]
    fn lock_unspent(&self, unlock: bool, outputs: Vec<JsonOutPoint>) -> RpcResult<bool>;

    #[method(name = "listdescriptors")]
    fn list_descriptors(
        &self,
        _with_private_keys: Option<bool>,
    ) -> RpcResult<ListDescriptorsResult>;

    #[method(name = "loadwallet")]
    fn load_wallet(&self, wallet: String) -> RpcResult<LoadWalletResult>;

    #[method(name = "listwallets")]
    fn list_wallets(&self) -> RpcResult<Vec<String>>;

    #[method(name = "listwalletdir")]
    fn list_wallet_dir(&self) -> RpcResult<ListWalletDirResult>;

    #[method(name = "walletprocesspsbt")]
    fn wallet_process_psbt(
        &self,
        psbt: String,
        sign: Option<bool>,
        sighash_type: Option<()>,
        bip32derivs: Option<bool>,
    ) -> RpcResult<WalletProcessPsbtResult>;

    #[method(name = "finalizepsbt")]
    fn finalize_psbt(&self, psbt: String, extract: Option<bool>) -> RpcResult<FinalizePsbtResult>;
}
