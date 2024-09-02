// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Network } from '../types/index.js'
import { Ordit, AddressFormats } from '@sadoprotocol/ordit-sdk'

export type WalletOptions = {
  wif?: string
  seed?: string
  privateKey?: string
  bip39?: string
  network?: Network
  type?: AddressFormats
}

export class Wallet extends Ordit {}
