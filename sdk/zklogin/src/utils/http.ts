// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export async function loadJSON(url: string): Promise<any> {
  const response = await fetch(url)
  return await response.json()
}
