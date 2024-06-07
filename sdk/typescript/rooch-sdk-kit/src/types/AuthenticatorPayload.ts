// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs, U8 } from '@roochnetwork/rooch-sdk'

export class AuthenticatorPayload implements bcs.Serializable {
  private readonly signature: Array<U8>
  private readonly messagePrefix: Array<U8>
  private readonly messageInfo: Array<U8>
  private readonly publicKey: Array<U8>
  private readonly fromAddress: Array<U8>

  constructor(
    signature: Array<U8>,
    messagePrefix: Array<U8>,
    messageInfo: Array<U8>,
    publicKey: Array<U8>,
    fromAddress: Array<U8>,
  ) {
    this.signature = signature
    this.messagePrefix = messagePrefix
    this.messageInfo = messageInfo
    this.publicKey = publicKey
    this.fromAddress = fromAddress
  }

  toBytes() {
    let bc = new bcs.BcsSerializer()
    this.serialize(bc)
    return bc.getBytes()
  }

  serialize(se: bcs.BcsSerializer) {
    bcs.Helpers.serializeVectorU8(this.signature, se)
    bcs.Helpers.serializeVectorU8(this.messagePrefix, se)
    bcs.Helpers.serializeVectorU8(this.messageInfo, se)
    bcs.Helpers.serializeVectorU8(this.publicKey, se)
    bcs.Helpers.serializeVectorU8(this.fromAddress, se)
  }
}
