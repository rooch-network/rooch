// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs, U8 } from '@roochnetwork/rooch-sdk'

export class AuthenticatorPayload implements bcs.Serializable {
  private readonly sign: Array<U8>
  private readonly signInfoPrefix: Array<U8>
  private readonly signInfo: Array<U8>
  private readonly publicKey: Array<U8>
  private readonly multiAddress: Array<U8>
  private readonly fromAddress: Array<U8>

  constructor(
    sign: Array<U8>,
    signInfoPrefix: Array<U8>,
    signInfo: Array<U8>,
    publicKey: Array<U8>,
    multiAddress: Array<U8>,
    fromAddress: Array<U8>,
  ) {
    this.sign = sign
    this.signInfoPrefix = signInfoPrefix
    this.signInfo = signInfo
    this.publicKey = publicKey
    this.multiAddress = multiAddress
    this.fromAddress = fromAddress
  }

  toBytes() {
    let bc = new bcs.BcsSerializer()
    this.serialize(bc)
    return bc.getBytes()
  }

  serialize(se: bcs.BcsSerializer) {
    bcs.Helpers.serializeVectorU8(this.sign, se)
    bcs.Helpers.serializeVectorU8(this.signInfoPrefix, se)
    bcs.Helpers.serializeVectorU8(this.signInfo, se)
    bcs.Helpers.serializeVectorU8(this.publicKey, se)
    bcs.Helpers.serializeVectorU8(this.multiAddress, se)
    bcs.Helpers.serializeVectorU8(this.fromAddress, se)
  }
}
