// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Args, type TypeTag } from '../bcs/index.js'

export type CallScript = {
  code: string
  args: Args[]
  typeArgs: TypeTag[]
}

type FunctionArgs =
  | {
      address: string
      module: string
      function: string
    }
  | {
      target: string
    }

export type CallFunctionArgs = {
  args?: Args[]
  typeArgs?: TypeTag[]
} & FunctionArgs

export type TypeArgs =
  | {
      address: string
      module: string
      name: string
    }
  | {
      target: string
    }
