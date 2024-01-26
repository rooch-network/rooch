// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export interface IPage<T, C> {
  data: Array<T>
  nextCursor: C | null
  hasNextPage: boolean
}
