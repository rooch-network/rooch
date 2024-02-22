// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BaseWallet } from './baseWallet'
import { RoochMultiChainID, SerializedSignature } from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'
import { MultiChainAddress } from '../address'
import { AuthenticatorPayload } from '../AuthenticatorPayload'

const BITCOIN_MAGIC_SIGN_PREFIX = 'Bitcoin Signed Message:\n'

export abstract class BitcoinWallet extends BaseWallet {
  protected toSerializedSignature(
    _: string,
    signature: string,
    signatureInfo: string,
  ): SerializedSignature {
    const walletAccount = this.account!

    let signBuffer = Buffer.from(signature, 'base64')

    // remove recover id
    const normalizeSignBuffer = signBuffer.subarray(1)

    let multiAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, walletAccount.getAddress())
    let multiAddressBytes = multiAddress.toBytes()
    let bitcoinMagicSignPrefixBytes = Array.from(BITCOIN_MAGIC_SIGN_PREFIX, (char) =>
      char.charCodeAt(0),
    )
    let signatureInfoBytes = Array.from(signatureInfo, (char) => char.charCodeAt(0))
    let publicKey = Buffer.from(walletAccount.getInfo().publicKey!, 'hex')

    let authPayload = new AuthenticatorPayload(
      Array.from(normalizeSignBuffer),
      bitcoinMagicSignPrefixBytes,
      signatureInfoBytes,
      Array.from(publicKey),
      Array.from(multiAddressBytes),
      [],
    )

    return authPayload.toBytes()
  }
  normalize_recovery_id(v: number) {
    let normalizeV = v - 27 - 4

    if (normalizeV < 0) {
      normalizeV = normalizeV + 4
    }

    return normalizeV
  }
}
