// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Seq, uint8, uint64 } from '../../generated/runtime/serde/mod'
import { BcsSerializer } from '../../generated/runtime/bcs/mod'
import { Helpers } from '../../generated/runtime/rooch_types/mod'
import { Serializable } from './serializable'

export class MultiChainAddress implements Serializable {
  private multichain_id: uint64
  private raw_address: Seq<uint8>

  public constructor(multichain_id: uint64, raw_address: Seq<uint8>) {
    this.multichain_id = multichain_id
    this.raw_address = raw_address
  }

  serialize(se: BcsSerializer): void {
    se.serializeU64(this.multichain_id)
    Helpers.serializeVectorU8(this.raw_address, se)
  }
}
