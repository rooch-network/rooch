// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export type Bytes = Uint8Array

export const EmptyBytes = new Uint8Array()

/**
 * Shortcut to one-element (element is 0) byte array
 */
export const NullBytes = new Uint8Array([0])
