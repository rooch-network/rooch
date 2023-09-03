// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Serializer, Deserializer } from '../../generated/runtime/serde/mod'
import { Seq, uint8 } from '../../generated/runtime/serde/mod'
import { Helpers } from '../../generated/runtime/rooch_types/mod'

export class MoveString {
  private bytes: Seq<uint8>

  constructor(bytes: Seq<uint8>) {
    this.bytes = bytes
  }

  public serialize(serializer: Serializer): void {
    Helpers.serializeVectorU8(this.bytes, serializer)
  }

  static deserialize(deserializer: Deserializer): MoveString {
    const bytes = Helpers.deserializeVectorU8(deserializer)
    return new MoveString(bytes)
  }
}
