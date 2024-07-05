// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import * as bitcoin from 'bitcoinjs-lib'

export const isValidBitcoinAddress = (address: string): boolean => {
  try {
    // Validate mainnet and testnet Bech32 addresses
    bitcoin.address.fromBech32(address)
    return true
  } catch (e) {
    try {
      // Validate mainnet and testnet base58 addresses
      bitcoin.address.fromBech32(address)
      return true
    } catch (e) {
      return false
    }
  }
}
