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
import { bcsTypes, Ed25519Keypair } from '@rooch/sdk'

// ** Defaults
const defaultProvider: AuthValuesType = {
  loading: true,
  setLoading: () => Boolean,
  accounts: null,
  supportWallets: [],
  defaultAccount: () => null,
  logout: () => Promise.resolve(),
  loginByWallet: () => Promise.resolve(),
  loginByNewAccount: () => Promise.resolve(),
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

      const secretKey = window.localStorage.getItem(authConfig.secretKey)

      if (secretKey) {
        console.log(secretKey)
        console.log(secretKey.length)
      }

      if (secretKey) {
        console.log(secretKey)
        let sk = bcsTypes.fromB64(secretKey)

        // The rooch cli generated key contains schema, remove it
        if (sk.length != 32) {
          sk = sk.slice(1)
        }

        const kp = Ed25519Keypair.fromSecretKey(sk)

        setAccountWrapper({
          address: kp.toRoochAddress(),
          kp: kp,
          activate: true,
          type: AccountType.ROOCH,
        })
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

  const setAccountWrapper = (account: AccountDataType) => {
    let _accounts = accounts
    if (!_accounts) {
      _accounts = new Map()
    }

    _accounts.set(account.address, {
      ...account,
    })

    setAccounts(_accounts)
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

    try {
      const sk = bcsTypes.fromB64(params.key)
      const kp = Ed25519Keypair.fromSecretKey(sk.slice(1))

      setAccountWrapper({
        address: kp.toRoochAddress(),
        kp: kp,
        activate: true,
        type: AccountType.ROOCH,
      })

      if (params.rememberMe) {
        window.localStorage.setItem(authConfig.secretKey, params.key)
      }
    }catch (e) {
      console.log(e)
      return
    }

    loginSuccess()
  }

  const loginByNewAccount = () => {
    const kp = Ed25519Keypair.generate()

    window.localStorage.setItem(authConfig.secretKey, kp.export().privateKey)

    setAccountWrapper({
      address: kp.toRoochAddress(),
      kp: kp,
      activate: true,
      type: AccountType.ROOCH,
    })

    loginSuccess()
  }

  const handleLogout = () => {
    window.localStorage.removeItem(authConfig.secretKey)
    setAccounts(null)
    metamask.disconnect()
  }

  const defaultAccount = (): AccountDataType => {
    return {
      address: 'aa',
      kp: null,
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
    defaultAccount,
    loginByWallet,
    loginBySecretKey,
    loginByNewAccount,
    logout: handleLogout,
  }

  return <AuthContext.Provider value={values}>{children}</AuthContext.Provider>
}

export { AuthContext, AuthProvider }
