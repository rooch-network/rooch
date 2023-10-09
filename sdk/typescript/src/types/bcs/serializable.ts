// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { BcsSerializer } from '../../generated/runtime/bcs/mod'

export interface Serializable {
  serialize(se: BcsSerializer): void
}
