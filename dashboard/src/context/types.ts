// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export type ErrCallbackType = (err: { [key: string]: string }) => void
export type Callback = () => void

export type AddAccountBySecretKeyParams = {
  key: string
  rememberMe?: boolean
}

export type AccountDataType = {
  address: string
  kp: string | null
  activate: boolean
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
