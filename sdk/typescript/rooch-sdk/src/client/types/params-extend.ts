// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export type PaginationArguments<Cursor> = {
  /** Optional paging cursor */
  cursor?: Cursor
  /** Maximum item returned per page */
  limit?: number | null
}

export type PaginationResult<Cursor, Result> = {
  cursor?: Cursor | null
  data: Array<Result>
  hasNextPage: boolean
}

export type SessionInfoView = {
  appName: string
  appUrl: string
  authenticationKey: string
  scopes: Array<string>
  createTime: number
  lastActiveTime: number
  maxInactiveInterval: number
}
