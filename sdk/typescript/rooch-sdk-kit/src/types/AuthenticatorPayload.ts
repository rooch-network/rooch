// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcsTypes, U8 } from '@roochnetwork/rooch-sdk'

export class AuthenticatorPayload implements bcsTypes.Serializable {
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
    let bcs = new bcsTypes.BcsSerializer()
    this.serialize(bcs)
    return bcs.getBytes()
  }

  serialize(se: bcsTypes.BcsSerializer) {
    bcsTypes.Helpers.serializeVectorU8(this.sign, se)
    bcsTypes.Helpers.serializeVectorU8(this.signInfoPrefix, se)
    bcsTypes.Helpers.serializeVectorU8(this.signInfo, se)
    bcsTypes.Helpers.serializeVectorU8(this.publicKey, se)
    bcsTypes.Helpers.serializeVectorU8(this.multiAddress, se)
    bcsTypes.Helpers.serializeVectorU8(this.fromAddress, se)
  }
}
