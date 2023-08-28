// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ErrCallbackType } from 'src/context/types'

export type AddAccountBySecretKeyParams = {
  key: string
  rememberMe?: boolean
}

export enum WalletType {
  Metamask = 'Metamask',
  Bitcoin = 'Bitcoin',
}

export enum AccountType {
  ETH,
  ROOCH,
}

export type SupportWalletType = {
  enable: boolean
  name: WalletType
}

export type AccountDataType = {
  address: string
  kp: string | null
  activate: boolean
  type: AccountType
}

export type AuthValuesType = {
  loading: boolean
  logout: () => void
  setLoading: (value: boolean) => void
  supportWallets: SupportWalletType[]
  accounts: Map<string, AccountDataType> | null
  addAccount: (value: AccountDataType | null) => void
  defaultAccount: () => AccountDataType | null
  loginByWallet: (walletType: WalletType, errorCallback?: ErrCallbackType) => void
  loginBySecretKey: (params: AddAccountBySecretKeyParams, errorCallback?: ErrCallbackType) => void
}
