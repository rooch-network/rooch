// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Ed25519Keypair } from '@roochnetwork/rooch-sdk'

export type ErrCallbackType = (err: { [key: string]: string }) => void
export type Callback = () => void

export type AddAccountBySecretKeyParams = {
  key: string
  rememberMe?: boolean
}

export enum AccountType {
  ETH = 'ETH',
  ROOCH = 'Rooch',
}

export type AccountDataType = {
  roochAddress: string
  address: string
  kp: Ed25519Keypair | null
  activate: boolean
  type: AccountType
}

export type AuthValuesType = {
  loading: boolean
  logout: () => void
  setLoading: (value: boolean) => void
  accounts: Map<string, AccountDataType> | null
  addAccount: (value: AccountDataType | null) => void
  defaultAccount: () => AccountDataType | null
  loginByMetamask: (errorCallback?: ErrCallbackType) => void
  loginByBitcoin: (errorCallback?: ErrCallbackType) => void
  loginBySecretKey: (params: AddAccountBySecretKeyParams, errorCallback?: ErrCallbackType) => void
}
