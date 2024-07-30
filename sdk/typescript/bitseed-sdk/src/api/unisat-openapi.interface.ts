import * as bitcoin from 'bitcoinjs-lib';

import {
  AddressSummary,
  AddressTokenSummary,
  AppSummary,
  Arc20Balance,
  BitcoinBalance,
  DecodedPsbt,
  FeeSummary,
  InscribeOrder,
  Inscription,
  InscriptionSummary,
  TokenBalance,
  TokenTransfer,
  UTXO,
  UTXO_Detail,
  VersionDetail,
  WalletConfig,
  Transaction
} from './unisat-openapi.types';

export interface IUniSatOpenAPI {
  getHost(): string;

  getNetwork(): bitcoin.Network;

  getWalletConfig(): Promise<WalletConfig>;

  getAddressSummary(address: string): Promise<AddressSummary>;

  getAddressBalance(address: string): Promise<BitcoinBalance>;

  getMultiAddressAssets(addresses: string): Promise<AddressSummary[]>;

  findGroupAssets(
    groups: { type: number; address_arr: string[] }[]
  ): Promise<{ type: number; address_arr: string[]; satoshis_arr: number[] }[]>;

  getBTCUtxos(address: string): Promise<UTXO[]>;

  getInscriptionUtxo(inscriptionId: string): Promise<UTXO>;

  getInscriptionUtxoDetail(inscriptionId: string): Promise<UTXO_Detail>;

  getInscriptionUtxos(inscriptionIds: string[]): Promise<UTXO[]>;

  getAddressInscriptions(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }>;

  getInscriptionSummary(): Promise<InscriptionSummary>;

  getAppSummary(): Promise<AppSummary>;

  getTx(txid: string): Promise<Transaction>;

  pushTx(rawtx: string): Promise<string>;

  getFeeSummary(): Promise<FeeSummary>;

  getDomainInfo(domain: string): Promise<Inscription>;

  inscribeBRC20Transfer(
    address: string,
    tick: string,
    amount: string,
    feeRate: number,
    outputValue: number
  ): Promise<InscribeOrder>;

  getInscribeResult(orderId: string): Promise<TokenTransfer>;

  getBRC20List(address: string, cursor: number, size: number): Promise<{ list: TokenBalance[]; total: number }>;

  getAddressTokenSummary(address: string, ticker: string): Promise<AddressTokenSummary>;

  getTokenTransferableList(
    address: string,
    ticker: string,
    cursor: number,
    size: number
  ): Promise<{ list: TokenTransfer[]; total: number }>;

  decodePsbt(psbtHex: string): Promise<DecodedPsbt>;

  createMoonpayUrl(address: string): Promise<string>;

  checkWebsite(website: string): Promise<{ isScammer: boolean; warning: string }>;

  getOrdinalsInscriptions(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }>;

  getAtomicalsNFT(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }>;

  getAtomicalsUtxo(atomicalId: string): Promise<UTXO>;

  getArc20BalanceList(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Arc20Balance[]; total: number }>;

  getArc20Utxos(address: string, ticker: string): Promise<UTXO[]>;

  getVersionDetail(version: string): Promise<VersionDetail>;

  loadContent(inscriptionid: string): Promise<ArrayBuffer>;
}
