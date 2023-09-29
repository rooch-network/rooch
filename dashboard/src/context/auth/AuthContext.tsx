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
import { bcsTypes, Ed25519Keypair, addressToSeqNumber } from '@rooch/sdk'

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
  // ** Hooks
  const metamask = useETH()
  const rooch = useRooch()

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

    console.log('resoleRoochAddress result:', result)

    if (result && result.vm_status === 'Executed' && result.return_values) {
      return result.return_values[0].move_value as string
    }

    throw new Error('resolve rooch address fail')
  }

  const updateETHAccount = async () => {
    if (metamask.accounts.length > 0) {
      const ethAddress = metamask.accounts[0]
      const roochAddress = await resoleRoochAddress(ethAddress)

      setAccountWrapper({
        address: ethAddress,
        roochAddress: roochAddress,
        activate: true,
        kp: null,
        type: AccountType.ETH,
      })
    }
  }

  const loginByWallet = (walletType: WalletType, errorCallback?: ErrCallbackType) => {
    switch (walletType) {
      case WalletType.Metamask:
        metamask
          .connect()
          .then(() => {
            updateETHAccount().then(() => {
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
    metamask.disconnect()
  }

  const defaultAccount = (): AccountDataType | null => {
    const accounts = getAccounts()

    if (accounts && accounts.size > 0) {
      return accounts.values().next().value
    }

    return null
  }

  const getAccounts = (): Map<string, AccountDataType> | null => {
    const allAccounts = accounts ?? new Map<string, AccountDataType>()

    return allAccounts.size > 0 ? allAccounts : null
  }

  const values = {
    loading: metamask.loading ?? loading ?? rooch.loading,
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
