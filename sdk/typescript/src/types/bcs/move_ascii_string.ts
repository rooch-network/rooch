// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Serializer, Deserializer } from '../../generated/runtime/serde/mod'
import { Seq, uint8 } from '../../generated/runtime/serde/mod'
import { Helpers } from '../../generated/runtime/rooch_types/mod'
import { Serializable } from './serializable'

export class MoveAsciiString implements Serializable {
  private bytes: Seq<uint8>

  constructor(bytes: Seq<uint8>) {
    this.bytes = bytes
  }

  public serialize(serializer: Serializer): void {
    Helpers.serializeVectorU8(this.bytes, serializer)
  }

  static deserialize(deserializer: Deserializer): MoveAsciiString {
    const bytes = Helpers.deserializeVectorU8(deserializer)
    return new MoveAsciiString(bytes)
  }
}
