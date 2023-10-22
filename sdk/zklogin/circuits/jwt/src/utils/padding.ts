// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function padString(str: string, paddedBytesSize: number): number[] {
  let paddedBytes = Array.from(str, (c) => c.charCodeAt(0))
  paddedBytes.push(...new Array(paddedBytesSize - paddedBytes.length).fill(0))
  return paddedBytes
}
