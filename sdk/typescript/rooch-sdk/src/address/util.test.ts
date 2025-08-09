// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair, Secp256k1Keypair } from '../keypairs/index.js'
import { isValidAddress } from './util.js'
import {bcs} from "../bcs";

describe('Address util', () => {
  it('Valid Rooch Address', () => {
    const ed25519RoochAddr = new Ed25519Keypair().getRoochAddress()

    expect(isValidAddress(ed25519RoochAddr)).toBeTruthy()
    expect(isValidAddress(ed25519RoochAddr.toHexAddress())).toBeTruthy()
    expect(isValidAddress(ed25519RoochAddr.toBech32Address())).toBeTruthy()

    const btcAddr = new Secp256k1Keypair().getBitcoinAddress()
    const roochAddr = btcAddr.genRoochAddress()

    expect(isValidAddress(btcAddr)).toBeTruthy()
    expect(isValidAddress(roochAddr)).toBeTruthy()
    expect(isValidAddress(roochAddr.toHexAddress())).toBeTruthy()
    expect(isValidAddress(roochAddr.toBech32Address())).toBeTruthy()
  })

  it('custom bsc address test', () => {
    const ChannelKeySchema = bcs.struct('ChannelKey', {
      sender: bcs.Address,
      receiver: bcs.Address,
      coin_type: bcs.string(),
    });

    const channelKey = {
      sender: '0x001',
      receiver: '0x0000000000000000000000000000000000000000000000000000000000000002',
      coin_type: '0x3::gas_coin::RGas',
    };

    const idBytes = ChannelKeySchema.serialize(channelKey).toBytes();

    console.log(idBytes)
  })

  it('should handle issue addresses correctly', () => {
    const ChannelKeySchema = bcs.struct('ChannelKey', {
      sender: bcs.Address,
      receiver: bcs.Address,
      coin_type: bcs.string(),
    });

    // Test the exact addresses from the issue
    const channelKey = {
      sender: '0x0000000000000000000000000000000000000000000000000000000000000001',
      receiver: '0x0000000000000000000000000000000000000000000000000000000000000002',
      coin_type: '0x3::gas_coin::RGas',
    };

    // This should not throw any errors
    expect(() => {
      const idBytes = ChannelKeySchema.serialize(channelKey).toBytes();
      console.log('Issue test result:', idBytes.length, 'bytes');
    }).not.toThrow();
  })
})
