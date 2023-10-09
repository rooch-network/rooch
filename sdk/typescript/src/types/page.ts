// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Bytes } from './bytes'

export interface IPage<T> {
  data: Array<T>
  nextCursor: Bytes | null
  hasNextPage: boolean
}
