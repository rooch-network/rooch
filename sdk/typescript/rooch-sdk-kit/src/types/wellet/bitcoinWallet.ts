// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BaseWallet } from './baseWallet'
import { SerializedSignature } from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'
import { AuthenticatorPayload } from '../AuthenticatorPayload'
import { SupportChain } from '../../feature'

const BITCOIN_MAGIC_MESSAGE_PREFIX = '\u0018Bitcoin Signed Message:\n'

export abstract class BitcoinWallet extends BaseWallet {
  protected toSerializedSignature(
    msg: string,
    signature: string,
    messageInfo: string,
  ): SerializedSignature {
    const walletAccount = this.account!

    // remove recover id
    let normalizeSignatureBuffer = Buffer.from(signature, 'base64').subarray(1)

    let messageInfoBuffer = Buffer.from(messageInfo)

    const bitcoinMagicMessagePrefixBuffer = Buffer.concat([
      Buffer.from(`${BITCOIN_MAGIC_MESSAGE_PREFIX}`, 'utf-8'),
      Buffer.from([messageInfoBuffer.length + msg.length]),
    ])

    let publicKey = Buffer.from(walletAccount.publicKey!, 'hex')

    let authPayload = new AuthenticatorPayload(
      Array.from(normalizeSignatureBuffer),
      Array.from(bitcoinMagicMessagePrefixBuffer),
      Array.from(messageInfoBuffer),
      Array.from(publicKey),
      Array.from(Buffer.from(walletAccount.address)),
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

  switchAccount(): void {
    throw new Error('Method not implemented.')
  }

  getChain(): SupportChain {
    return SupportChain.BITCOIN
  }
}
