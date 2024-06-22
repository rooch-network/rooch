// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'

export function validateWitness(version: number, data: Bytes) {
  if (data.length < 2 || data.length > 40) throw new Error('Witness: invalid length')
  if (version > 16) throw new Error('Witness: invalid version')
  if (version === 0 && !(data.length === 20 || data.length === 32))
    throw new Error('Witness: invalid length for version')
}
