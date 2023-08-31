// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, ReactNode, useEffect, useState } from 'react'

// ** Next Import
import { useRouter } from 'next/router'

// ** Config
import authConfig from 'src/configs/auth'

// ** Types
import {
  AccountDataType,
  AccountType,
  AddAccountBySecretKeyParams,
  AuthValuesType,
  SupportWalletType,
  WalletType,
} from 'src/context/auth/types'

import { ErrCallbackType } from 'src/context/types'

// ** Hooks
import { useMetamask } from 'src/hooks/useMetamask'

// ** Rooch SDK
import { Ed25519Keypair } from '@rooch/sdk'

// ** Defaults
const defaultProvider: AuthValuesType = {
  loading: true,
  setLoading: () => Boolean,
  accounts: null,
  supportWallets: [],
  addAccount: () => null,
  defaultAccount: () => null,
  logout: () => Promise.resolve(),
  loginByWallet: () => Promise.resolve(),
  loginBySecretKey: () => Promise.resolve(),
}

const AuthContext = createContext(defaultProvider)

type Props = {
  children: ReactNode
}

const AuthProvider = ({ children }: Props) => {
  const metamask = useMetamask()

  // ** States
  const [accounts, setAccounts] = useState<Map<string, AccountDataType> | null>(
    defaultProvider.accounts,
  )
  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  // ** Hooks
  const router = useRouter()

  useEffect(() => {
    const initAuth = async (): Promise<void> => {
      setLoading(true)

      const kp = Ed25519Keypair.generate()

      console.log(kp.getPublicKey().toRoochAddress())

      const allSecretKey = window.localStorage.getItem(authConfig.secretKey)

      if (allSecretKey) {
        // TODO: Parse key
        const acc = new Map<string, AccountDataType>()
        acc.set('0x12345', {
          address: '0x12345',
          kp: null,
          activate: true,
          type: AccountType.ROOCH,
        })
        setAccounts(acc)
      }
    }

    initAuth().finally(() => setLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const loginSuccess = () => {
    const returnUrl = router.query.returnUrl

    const redirectURL = returnUrl && returnUrl !== '/' ? returnUrl : '/'

    router.replace(redirectURL as string)
  }

  const tmpLogin = () => {
    setAccounts(new Map())

    window.localStorage.setItem(authConfig.secretKey, '000')

    loginSuccess()
  }

  /// ** Impl fun
  const supportWallets = (): SupportWalletType[] => {
    const result: SupportWalletType[] = []
    for (const key in WalletType) {
      switch (WalletType[key as keyof typeof WalletType]) {
        case WalletType.Metamask:
          result.push({
            enable: metamask.hasProvider,
            name: WalletType.Metamask,
          })
          break
        default:
          result.push({
            enable: false,
            name: WalletType[key as keyof typeof WalletType],
          })
          break
      }
    }

    if (result.some((value) => value.enable)) {
      return result
    }

    return []
  }

  const loginByWallet = (walletType: WalletType, errorCallback?: ErrCallbackType) => {
    switch (walletType) {
      case WalletType.Metamask:
        metamask
          .connect()
          .then(loginSuccess)
          .catch((e) => {
            if (errorCallback) {
              errorCallback(e)
            }
          })
        break
    }
  }

  const loginBySecretKey = (params: AddAccountBySecretKeyParams) => {
    // TODO: use rooch sdk
    console.log(params)
    tmpLogin()
  }

  const addAccount = () => {
    tmpLogin()
  }

  const handleLogout = () => {
    window.localStorage.removeItem(authConfig.secretKey)
    setAccounts(null)
    metamask.disconnect()
  }

  const defaultAccount = (): AccountDataType => {
    return {
      address: 'aa',
      kp: 'aa',
      activate: true,
      type: AccountType.ROOCH,
    }
  }

  const getAccounts = (): Map<string, AccountDataType> | null => {
    const allAccounts = accounts ?? new Map<string, AccountDataType>()

    // TODO: abstract wallet
    if (metamask.accounts.length > 0) {
      metamask.accounts.forEach((v) => {
        allAccounts.set(v, {
          address: v,
          activate: true,
          kp: null,
          type: AccountType.ETH,
        })
      })
    }

    return allAccounts.size > 0 ? allAccounts : null
  }

  const values = {
    loading: metamask.loading ?? loading,
    setLoading,
    accounts: getAccounts(),
    setAccounts,
    supportWallets: supportWallets(),
    addAccount,
    defaultAccount,
    loginByWallet,
    loginBySecretKey,
    logout: handleLogout,
  }

  return <AuthContext.Provider value={values}>{children}</AuthContext.Provider>
}

export { AuthContext, AuthProvider }
