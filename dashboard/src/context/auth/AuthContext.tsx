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
import { useETH } from 'src/hooks/useETH'
import { useRooch } from '../../hooks/useRooch'

// ** Rooch SDK
import { addressToSeqNumber, bcsTypes, Ed25519Keypair } from '@rooch/sdk'

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
  const metamask = useETH()
  const rooch = useRooch()

  // ** States
  const [roochAddressMap, setRoochAddressMap] = useState<Map<string, string>>(new Map())
  const [defaultAccount, setDefaultAccount] = useState<AccountDataType | null>(() => {
    if (defaultProvider.accounts && defaultProvider.accounts.size > 0) {
      return defaultProvider.accounts.values().next().value
    }

    return null
  })

  const [accounts, setAccounts] = useState<Map<string, AccountDataType> | null>(
    defaultProvider.accounts,
  )

  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  // ** Hooks
  const router = useRouter()

  useEffect(() => {
    const initAuth = async (): Promise<void> => {
      setLoading(true)

      const roochAddressMapStr = window.localStorage.getItem(authConfig.roochAccountMap)

      if (roochAddressMapStr) {
        setRoochAddressMap(new Map<string, string>(JSON.parse(roochAddressMapStr)))
      }

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

  const resoleRoochAddress = async (ethAddress: string): Promise<string> => {
    const multiChainIDEther = 60

    const ma = new bcsTypes.MultiChainAddress(
      BigInt(multiChainIDEther),
      addressToSeqNumber(ethAddress),
    )

    const result = await rooch?.provider?.executeViewFunction(
      '0x3::address_mapping::resolve_or_generate',
      [],
      [
        {
          type: {
            Struct: {
              address: '0x3',
              module: 'address_mapping',
              name: 'MultiChainAddress',
            },
          },
          value: ma,
        },
      ],
    )

    if (result && result.vm_status === 'Executed' && result.return_values) {
      return result.return_values[0].decoded_value as string
    }

    throw new Error('resolve rooch address fail')
  }

  const updateETHAccount = async (account?: string[]) => {
    const _account = account ?? metamask.accounts
    if (_account.length > 0) {
      const ethAddress = _account[0]
      const roochAddress = await resoleRoochAddress(ethAddress)

      setAccountWrapper({
        address: ethAddress,
        roochAddress: roochAddress,
        activate: true,
        kp: null,
        type: AccountType.ETH,
      })

      // TODO: clear
      roochAddressMap.set(ethAddress, roochAddress)

      window.localStorage.setItem(
        authConfig.roochAccountMap,
        JSON.stringify(Array.from(roochAddressMap.entries())),
      )
    }
  }

  const loginByWallet = (walletType: WalletType, errorCallback?: ErrCallbackType) => {
    switch (walletType) {
      case WalletType.Metamask:
        metamask
          .connect()
          .then((v: any) => {
            updateETHAccount(v).then(() => {
              loginSuccess && loginSuccess()
            })
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
    setAccounts(null)

    // TODO: wait fix in next metamask sdk
    // metamask.disconnect()
  }

  const getAccounts = (): Map<string, AccountDataType> | null => {
    const allAccounts = accounts ?? new Map<string, AccountDataType>()

    // Todo Parse the rooch address
    if (metamask.accounts.length > 0) {
      metamask.accounts.forEach((v) => {
        allAccounts.set(v, {
          roochAddress: roochAddressMap.get(v) ?? v,
          address: v,
          activate: true,
          kp: null,
          type: AccountType.ETH,
        })
      })
    }

    console.log('all acc', allAccounts)

    return allAccounts.size > 0 ? allAccounts : null
  }

  const getDefaultAccount = (): AccountDataType | null => {
    if (defaultAccount) {
      return defaultAccount
    }

    if (metamask.accounts.length > 0) {
      const account = metamask.accounts[0]

      return {
        roochAddress: roochAddressMap.get(account) ?? account,
        address: account,
        kp: null,
        activate: true,
        type: AccountType.ETH,
      }
    }

    return null
  }

  const values = {
    loading: metamask.loading ?? loading ?? rooch.loading,
    setLoading,
    accounts: getAccounts(),
    setAccounts,
    supportWallets: supportWallets(),
    defaultAccount: getDefaultAccount(),
    loginByWallet,
    loginBySecretKey,
    loginByNewAccount,
    logout: handleLogout,
  }

  return <AuthContext.Provider value={values}>{children}</AuthContext.Provider>
}

export { AuthContext, AuthProvider }
