// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { blake2b } from '@noble/hashes/blake2b'
import { ripemd160 } from '@noble/hashes/ripemd160'
import { sha3_256 } from '@noble/hashes/sha3'
import { sha256 } from '@noble/hashes/sha256'
import { sha512 } from '@noble/hashes/sha512'
import { utils as packedUtils } from 'micro-packed'

import { Bytes } from '@/types'

const { concatBytes } = packedUtils

export { sha256 }
export { sha3_256 }
export { sha512 }
export { blake2b }

export const hash160 = (msg: Bytes) => ripemd160(sha256(msg))
export const sha256x2 = (...msgs: Bytes[]) => sha256(sha256(concatBytes(...msgs)))
