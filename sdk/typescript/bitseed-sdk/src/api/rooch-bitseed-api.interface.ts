// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { SFTRecord } from '../types/index.js'

export interface RoochBitSeedApiInterface {
  getBitSeedSFTByID(): Promise<SFTRecord>
}
