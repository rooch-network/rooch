// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAuthorizer } from '../auth'

export interface IAccount {
  getAuthorizer(): IAuthorizer
  getAddress(): string
}
