// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { runtime } from '../index'

export interface IAuthorizer {
  auth(payload: Uint8Array, authInfo?: string): Promise<runtime.Authenticator>
}
