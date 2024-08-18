// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { InscriptionID } from './generator.js'

export type Tick = {
  tick: string
  max: number
  generator: InscriptionID
  repeat: number
  deploy_args: Array<string>
}
