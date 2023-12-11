// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { AnyAction, Dispatch } from '@reduxjs/toolkit'
import { RoochClient } from '@roochnetwork/rooch-sdk'

export interface Params {
  dispatch: Dispatch<AnyAction>
  provider: RoochClient
  accessPath: string
}
