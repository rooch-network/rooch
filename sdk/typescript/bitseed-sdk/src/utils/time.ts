// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
