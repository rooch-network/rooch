// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

declare module 'base58-js'
declare module 'sha256-uint8array'

declare function createHash(algorithm?: string): Hash

declare class Hash {
  update(data: string, encoding?: string): this
  update(data: Uint8Array): this
  update(data: ArrayBufferView): this

  digest(): Uint8Array
}
