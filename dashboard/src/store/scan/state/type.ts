// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { AnyAction, Dispatch } from '@reduxjs/toolkit'
import { JsonRpcProvider } from '@rooch/sdk'

export interface Params {
  dispatch: Dispatch<AnyAction>
  provider: JsonRpcProvider
  accessPath: string
}
