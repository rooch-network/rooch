// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Bytes } from '../types'

export interface IAuthorization {
  scheme: number
  payload: Bytes
}

export interface IAuthorizer {
  auth(callData: Bytes): Promise<IAuthorization>
}
