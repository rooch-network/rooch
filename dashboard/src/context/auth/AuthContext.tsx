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
  AddAccountBySecretKeyParams,
  AuthValuesType,
  SupportWalletType,
  WalletType,
} from 'src/context/auth/types'

import { AccountDataType, AccountType, ErrCallbackType } from 'src/context/types'

// ** Hooks
import { useETH } from 'src/hooks/useETH'
import { useRooch } from '../../hooks/useRooch'

// ** Rooch SDK
import { bcsTypes, Ed25519Keypair } from '@rooch/sdk'

// ** Defaults
const defaultProvider: AuthValuesType = {
  loading: true,
  setLoading: () => Boolean,
  accounts: null,
  supportWallets: [],
  defaultAccount: null,
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
  // ** Hooks
  const eth = useETH()
  const rooch = useRooch()

  // ** States
  const [defaultAccount, setDefaultAccount] = useState(defaultProvider.defaultAccount)
  const [accounts, setAccounts] = useState(defaultProvider.accounts)

  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  // ** Hooks
  const router = useRouter()

  useEffect(() => {
    const initAuth = async (): Promise<void> => {
      setLoading(true)

      const secretKey = window.localStorage.getItem(authConfig.secretKey)

      if (secretKey) {
        let sk = bcsTypes.fromB64(secretKey)

        // The rooch cli generated key contains schema, remove it
        if (sk.length > 32) {
          sk = sk.slice(1)
        }

        const kp = Ed25519Keypair.fromSecretKey(sk)
        const roochAddress = kp.toRoochAddress()

        setAccountWrapper({
          address: roochAddress,
          roochAddress: roochAddress,
          kp: kp,
          activate: true,
          type: AccountType.ROOCH,
        })
      }
    }

    initAuth().finally(() => setLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    if (eth.activeAccount) {
      setDefaultAccount(eth.activeAccount)
    }
  }, [eth.activeAccount])

  useEffect(() => {
    if (!eth.isConnect) {
      return
    }

    const roochAccount = new Map()

    accounts?.forEach((v) => {
      if (v.type === AccountType.ROOCH) {
        roochAccount.set(v.address, v)
      }
    })

    let _accounts = new Map<string, AccountDataType>(roochAccount)

    if (eth.accounts && eth.accounts.size > 0) {
      _accounts = new Map([..._accounts, ...eth.accounts])
    }

    if (_accounts.size > 0) {
      setAccounts(_accounts)
    } else {
      setAccounts(null)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eth.accounts, eth.isConnect])

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

    setAccounts(new Map([..._accounts]))

    if (_accounts && _accounts.size > 0) {
      setDefaultAccount(_accounts.values().next().value)
    }

    return null
  }

  /// ** Impl fun
  const supportWallets = (): SupportWalletType[] => {
    const result: SupportWalletType[] = []
    for (const key in WalletType) {
      switch (WalletType[key as keyof typeof WalletType]) {
        case WalletType.Metamask:
          result.push({
            enable: eth.hasProvider,
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
    window.localStorage.removeItem(authConfig.roochAccountMap)
    window.sessionStorage.clear()
    switch (walletType) {
      case WalletType.Metamask:
        eth
          .connect()
          .then((v: any) => {
            loginSuccess && loginSuccess()
          })
          .catch((e: any) => {
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

      const kp = Ed25519Keypair.fromSecretKey(sk)
      const roochAddress = kp.toRoochAddress()

      setAccountWrapper({
        address: roochAddress,
        roochAddress: roochAddress,
        kp: kp,
        activate: true,
        type: AccountType.ROOCH,
      })

      if (params.rememberMe) {
        window.localStorage.setItem(authConfig.secretKey, params.key)
      }
    } catch (e) {
      console.log(e)

      return
    }

    loginSuccess()
  }

  const loginByNewAccount = () => {
    const kp = Ed25519Keypair.generate()

    window.localStorage.setItem(authConfig.secretKey, kp.export().privateKey)

    const roochAddress = kp.toRoochAddress()

    setAccountWrapper({
      address: roochAddress,
      roochAddress: roochAddress,
      kp: kp,
      activate: true,
      type: AccountType.ROOCH,
    })

    loginSuccess()
  }

  const handleLogout = () => {
    window.localStorage.removeItem(authConfig.secretKey)

    let _accounts = accounts ?? new Map<string, AccountDataType>()

    const toDel: string[] = []
    _accounts.forEach((v) => {
      if (v.type === AccountType.ROOCH) {
        toDel.push(v.address)
      }
    })

    toDel.forEach((v) => {
      _accounts.delete(v)
    })

    if (eth.accounts && eth.accounts.size > 0) {
      _accounts = new Map([..._accounts, ...eth.accounts])
    }

    if (_accounts.size > 0) {
      setAccounts(_accounts)
    } else {
      setAccounts(null)
    }

    // TODO: wait fix in next metamask sdk
    // metamask.disconnect()
  }

  const values = {
    loading: eth.loading ? eth.loading : loading ? loading : rooch.loading,
    setLoading,
    accounts: accounts !== null ? accounts : eth.accounts.size > 0 ? eth.accounts : null, // Ensure that the eth account can be accessed in the first frame
    setAccounts,
    supportWallets: supportWallets(),
    defaultAccount: defaultAccount,
    loginByWallet,
    loginBySecretKey,
    loginByNewAccount,
    logout: handleLogout,
  }

  return <AuthContext.Provider value={values}>{children}</AuthContext.Provider>
}

export { AuthContext, AuthProvider }
