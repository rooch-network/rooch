// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** Next Import
import { useRouter } from 'next/router'

// ** Config
import authConfig from 'src/configs/auth'

// ** Types
import { AuthValuesType, AddAccountBySecretKeyParams, AccountDataType } from './types'

// ** Defaults
const defaultProvider: AuthValuesType = {
  loading: true,
  setLoading: () => Boolean,
  accounts: null,
  addAccount: () => null,
  defaultAccount: () => null,
  logout: () => Promise.resolve(),
  loginByBitcoin: () => Promise.resolve(),
  loginByMetamask: () => Promise.resolve(),
  loginBySecretKey: () => Promise.resolve()
}

const AuthContext = createContext(defaultProvider)

type Props = {
  children: ReactNode
}

const AuthProvider = ({ children }: Props) => {
  // ** States
  const [accounts, setAccounts] = useState<Map<string, AccountDataType> | null>(defaultProvider.accounts)
  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  // ** Hooks
  const router = useRouter()

  useEffect(() => {
    const initAuth = async (): Promise<void> => {
      const allSecretKey = window.localStorage.getItem(authConfig.secretKey)

      if (allSecretKey) {
        setLoading(true)

        // TODO: Parse key
        setAccounts(new Map())

        setLoading(false)
      } else {
        setLoading(false)
      }
    }

    initAuth()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const tmpLogin = () => {
    setAccounts(new Map())

    window.localStorage.setItem(authConfig.secretKey, '000')

    const returnUrl = router.query.returnUrl

    const redirectURL = returnUrl && returnUrl !== '/' ? returnUrl : '/'

    router.replace(redirectURL as string)
  }

  const loginByBitcoin = () => {
    console.log('TODO:')
    tmpLogin()
  }

  const loginByMetamask = () => {
    console.log('TODO:')
    tmpLogin()
  }

  const loginBySecretKey = (params: AddAccountBySecretKeyParams) => {
    console.log(params)
    console.log('TODO:')
    tmpLogin()
  }

  const addAccount = () => {
    console.log('TODO:')
    tmpLogin()
  }

  const handleLogout = () => {
    setAccounts(null)
    window.localStorage.removeItem(authConfig.secretKey)
    router.push('/login')
  }

  const defaultAccount = (): AccountDataType => {
    return {
      address: 'aa',
      kp: 'aa',
      activate: true
    }
  }

  const values = {
    loading,
    setLoading,
    accounts,
    setAccounts,
    addAccount,
    defaultAccount,
    loginByBitcoin,
    loginByMetamask,
    loginBySecretKey,
    logout: handleLogout
  }

  return <AuthContext.Provider value={values}>{children}</AuthContext.Provider>
}

export { AuthContext, AuthProvider }
