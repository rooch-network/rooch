// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export type Address = string // Assuming address is represented as a hex string

export interface OutPoint {
  txid: Address
  vout: number
}

export interface Witness {
  witness: string[]
}

export interface TxIn {
  previous_output: OutPoint
  script_sig: string // Assuming this is a base64 encoded string
  sequence: number
  witness: Witness
}

export interface TxOut {
  value: string // Using string to represent u64
  script_pubkey: string // Assuming this is a base64 encoded string
  recipient_address: string // Assuming this is the BitcoinAddress as a string
}

export interface RoochBTCTransaction {
  id: Address
  version: number
  lock_time: number
  input: TxIn[]
  output: TxOut[]
  metadata?: string // Optional, assuming this is a base64 encoded string
}
