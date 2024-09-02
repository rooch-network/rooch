// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export enum CHAINS_ENUM {
  BTC = 'BTC',
}

export enum AddressType {
  P2PKH,
  P2WPKH,
  P2TR,
  P2SH_P2WPKH,
  M44_P2WPKH,
  M44_P2TR,
}

export enum NetworkType {
  MAINNET,
  TESTNET,
  REGTEST,
}

export enum RestoreWalletType {
  UNISAT,
  SPARROW,
  XVERSE,
  OW,
  OTHERS,
}

export interface Chain {
  name: string
  logo: string
  enum: CHAINS_ENUM
  network: string
}

export interface BitcoinBalance {
  confirm_amount: string
  pending_amount: string
  amount: string
  confirm_btc_amount: string
  pending_btc_amount: string
  btc_amount: string
  confirm_inscription_amount: string
  pending_inscription_amount: string
  inscription_amount: string
  usd_value: string
}

export interface AddressAssets {
  total_btc: string
  satoshis?: number
  total_inscription: number
}

export interface TxHistoryItem {
  txid: string
  time: number
  date: string
  amount: string
  symbol: string
  address: string
}

export interface Inscription {
  inscriptionId: string
  inscriptionNumber: number
  address: string
  outputValue: number
  preview: string
  content: string
  contentType: string
  contentLength: number
  timestamp: number
  genesisTransaction: string
  location: string
  output: string
  offset: number
  contentBody: string
  utxoHeight: number
  utxoConfirmation: number
}

export interface Atomical {
  atomicalId: string
  atomicalNumber: number
  type: 'FT' | 'NFT'
  ticker?: string

  // mint info
  address: string
  outputValue: number
  preview: string
  content: string
  contentType: string
  contentLength: number
  timestamp: number
  genesisTransaction: string
  location: string
  output: string
  offset: number
  contentBody: string
  utxoHeight: number
  utxoConfirmation: number
}

export interface InscriptionMintedItem {
  title: string
  desc: string
  inscriptions: Inscription[]
}

export interface InscriptionSummary {
  mintedList: InscriptionMintedItem[]
}

export interface AppInfo {
  logo: string
  title: string
  desc: string
  url: string
  time: number
  id: number
  tag?: string
  readtime?: number
  new?: boolean
  tagColor?: string
}

export interface AppSummary {
  apps: AppInfo[]
  readTabTime?: number
}

export interface FeeSummary {
  list: {
    title: string
    desc: string
    feeRate: number
  }[]
}

export interface UTXO {
  txid: string
  vout: number
  satoshis: number
  scriptPk: string
  addressType: AddressType
  inscriptions: {
    inscriptionId: string
    inscriptionNumber?: number
    offset: number
  }[]
  atomicals: {
    atomicalId: string
    atomicalNumber: number
    type: 'NFT' | 'FT'
    ticker?: string
  }[]
}

export interface UTXO_Detail {
  txId: string
  outputIndex: number
  satoshis: number
  scriptPk: string
  addressType: AddressType
  inscriptions: Inscription[]
}

export enum TxType {
  SIGN_TX,
  SEND_BITCOIN,
  SEND_ORDINALS_INSCRIPTION,
  SEND_ATOMICALS_INSCRIPTION,
}

interface BaseUserToSignInput {
  index: number
  sighashTypes: number[] | undefined
  disableTweakSigner?: boolean
}

export interface AddressUserToSignInput extends BaseUserToSignInput {
  address: string
}

export interface PublicKeyUserToSignInput extends BaseUserToSignInput {
  publicKey: string
}

export type UserToSignInput = AddressUserToSignInput | PublicKeyUserToSignInput

export interface SignPsbtOptions {
  autoFinalized: boolean
  toSignInputs?: UserToSignInput[]
}

export interface ToSignInput {
  index: number
  publicKey: string
  sighashTypes?: number[]
}
export type WalletKeyring = {
  key: string
  index: number
  type: string
  addressType: AddressType
  accounts: Account[]
  alianName: string
  hdPath: string
}

export interface Account {
  type: string
  pubkey: string
  address: string
  brandName?: string
  alianName?: string
  displayBrandName?: string
  index?: number
  balance?: number
  key: string
  flag: number
}

export interface InscribeOrder {
  orderId: string
  payAddress: string
  totalFee: number
  minerFee: number
  originServiceFee: number
  serviceFee: number
  outputValue: number
}

export interface TokenBalance {
  availableBalance: string
  overallBalance: string
  ticker: string
  transferableBalance: string
  availableBalanceSafe: string
  availableBalanceUnSafe: string
}

export interface Arc20Balance {
  ticker: string
  balance: number
  confirmedBalance: number
  unconfirmedBalance: number
}

export interface TokenInfo {
  totalSupply: string
  totalMinted: string
  decimal: number
}

export enum TokenInscriptionType {
  INSCRIBE_TRANSFER,
  INSCRIBE_MINT,
}
export interface TokenTransfer {
  ticker: string
  amount: string
  inscriptionId: string
  inscriptionNumber: number
  timestamp: number
}

export interface AddressTokenSummary {
  tokenInfo: TokenInfo
  tokenBalance: TokenBalance
  historyList: TokenTransfer[]
  transferableList: TokenTransfer[]
}

export interface DecodedPsbt {
  inputInfos: {
    txid: string
    vout: number
    address: string
    value: number
    inscriptions: Inscription[]
    atomicals: Atomical[]
    sighashType: number
  }[]
  outputInfos: {
    address: string
    value: number
    inscriptions: Inscription[]
    atomicals: Atomical[]
  }[]
  inscriptions: { [key: string]: Inscription }
  feeRate: number
  fee: number
  features: {
    rbf: boolean
  }
  risks: { level: 'high' | 'low'; desc: string }[]
}

export interface ToAddressInfo {
  address: string
  domain?: string
  inscription?: Inscription
}

export interface RawTxInfo {
  psbtHex: string
  rawtx: string
  toAddressInfo?: ToAddressInfo
  fee?: number
}

export interface WalletConfig {
  version: string
  moonPayEnabled: boolean
  statusMessage: string
}

export enum WebsiteState {
  CHECKING,
  SCAMMER,
  SAFE,
}

export interface AddressSummary {
  totalSatoshis: number
  btcSatoshis: number
  assetSatoshis: number
  inscriptionCount: number
  atomicalsCount: number
  brc20Count: number
  arc20Count: number
  loading?: boolean
}

export interface VersionDetail {
  version: string
  title: string
  changelogs: string[]
}

export declare enum RarityEnum {
  COMMON = 'common',
  UNCOMMON = 'uncommon',
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic',
}
export type Rarity = `${RarityEnum}`
export interface Ordinal {
  number: number
  decimal: string
  degree: string
  name: string
  height: number
  cycle: number
  epoch: number
  period: number
  offset: number
  rarity: Rarity
  output: string
  start: number
  size: number
}

export interface Inscription {
  id: string
  outpoint: string
  owner: string
  genesis: string
  fee: number
  height: number
  number: number
  sat: number
  timestamp: number
  mediaType: string
  mediaSize: number
  mediaContent: string
  meta?: Record<string, any>
  value?: number
}

export type Vout = {
  value: number
  n: number
  ordinals: Ordinal[]
  inscriptions: Inscription[]
  spent: string | false
  sats: number
  scriptPubKey: {
    asm: string
    desc: string
    hex: string
    reqSigs?: number
    type: string
    addresses?: string[]
    address?: string
  }
}
export type Vin = {
  txid: string
  vout: number
  scriptSig: {
    asm: string
    hex: string
  }
  txinwitness?: string[]
  sequence: number
  value: number
}

export interface Transaction {
  hex?: string
  txid: string
  hash: string
  size: number
  vsize: number
  version: number
  locktime: number
  vin: Vin[]
  vout: Vout[]
  blockhash: string
  blockheight: number
  blocktime: number
  confirmations: number
  time: number
  weight: number
  fee: number
}
