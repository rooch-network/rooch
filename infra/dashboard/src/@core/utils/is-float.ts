// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const isFloat = (n: number) => {
  return typeof n === 'number' && n % 1 !== 0
}
