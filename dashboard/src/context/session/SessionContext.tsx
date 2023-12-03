// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { hexlify } from '@ethersproject/bytes'

// ** React Imports
import { createContext, ReactNode, useEffect, useMemo, useState } from 'react'
import { useAuth } from 'src/hooks/useAuth'
import { useRooch } from 'src/hooks/useRooch'

import { AccountDataType, AccountType } from 'src/context/types'

// ** Rooch SDK
import {
  Account,
  addressToSeqNumber,
  bcsTypes,
  Ed25519Keypair,
  encodeMoveCallData,
  ErrorCategory,
  FilteredProvider,
  FilterFunc,
  FuncFilter,
  getErrorCategoryName,
  IAccount,
  IClient,
  ITransactionFilterChain,
  parseRoochErrorSubStatus,
  PrivateKeyAuth,
} from '@roochnetwork/rooch-sdk'
import { Session } from 'src/context/session/types'
import { useETH } from '../../hooks/useETH'

type Props = {
  children: ReactNode
}

const SessionContext = createContext<Session>({
  loading: false,
  account: null,
  errorMsg: null,
  defaultSession: '',
  initialization: true,
  requestAuthorize: undefined,
  close: () => {},
})

const makeSessionAccountStoreKey = (chainId: number, address: string) => {
  return `rooch::${chainId}::dashboard::account::${address}::current-session-key`
}

const loadSessionAccountFromSessionStorage = (provider: IClient, roochAddress: string) => {
  try {
    // Get from local storage by key
    const secretKey = window.sessionStorage.getItem(
      makeSessionAccountStoreKey(provider.getChainId(), roochAddress),
    )

    if (secretKey) {
      let sk = bcsTypes.fromB64(secretKey)

      // The rooch cli generated key contains schema, remove it
      if (sk.length > 32) {
        sk = sk.slice(1)
      }

      const pk = Ed25519Keypair.fromSecretKey(sk)
      const authorizer = new PrivateKeyAuth(pk)

      return new Account(provider, roochAddress, authorizer)
    }
  } catch (error) {
    // If error also return initialValue
    console.log(error)
  }

  return null
}

const clearSessionAccountInSessionStorage = (provider: IClient, roochAddress: string) => {
  try {
    window.sessionStorage.setItem(
      makeSessionAccountStoreKey(provider.getChainId(), roochAddress),
      '',
    )
  } catch (error) {
    // If error also return initialValue
    console.log(error)
  }

  return null
}

const SessionProvider = ({ children }: Props) => {
  const auth = useAuth()
  const rooch = useRooch()
  const eth = useETH()

  const [loading, setLoading] = useState<boolean>(false)
  const [initialization, setInitialization] = useState(true)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)

  const filterdProvider = useMemo(() => {
    const sessionKeyInvalidFilterFunc: FilterFunc = async (
      req: any,
      chain: ITransactionFilterChain,
    ): Promise<any> => {
      try {
        return await chain.doFilter(req)
      } catch (e: any) {
        console.log('sessionKeyInvalidFilterFunc catch error:', e)
        const subStatus = parseRoochErrorSubStatus(e.message)
        if (
          subStatus &&
          ((subStatus.category === ErrorCategory.INVALID_ARGUMENT && subStatus.reason === 1001) ||
            subStatus.category === ErrorCategory.PERMISSION_DENIED)
        ) {
          setSessionAccount(null)

          const defaultAccount = auth.defaultAccount
          if (defaultAccount) {
            clearSessionAccountInSessionStorage(rooch.provider!, defaultAccount.roochAddress)
          }
        }

        throw e
      }
    }

    return new FilteredProvider(rooch.provider!, [new FuncFilter(sessionKeyInvalidFilterFunc)])
  }, [rooch.provider, auth.defaultAccount])

  const [sessionAccount, setSessionAccount] = useState<IAccount | null>(null)

  useEffect(() => {
    // TODO: add new dialog show get roochAddress
    if (!auth.defaultAccount || !auth.defaultAccount?.roochAddress) {
      return
    }

    setInitialization(true)

    const sessionAccount = loadSessionAccountFromSessionStorage(
      filterdProvider,
      auth.defaultAccount.roochAddress,
    )

    setSessionAccount(sessionAccount)

    setInitialization(false)
  }, [auth.defaultAccount, filterdProvider])

  const registerSessionKey = async (
    account: string,
    authKey: string,
    scopes: Array<string>,
    maxInactiveInterval: number,
  ) => {
    const [scopeModuleAddresss, scopeModuleNames, scopeFunctionNames] = scopes
      .map((scope: string) => {
        const parts = scope.split('::')
        if (parts.length !== 3) {
          throw new Error('invalid scope')
        }

        const scopeModuleAddress = parts[0]
        const scopeModuleName = parts[1]
        const scopeFunctionName = parts[2]

        return [scopeModuleAddress, scopeModuleName, scopeFunctionName]
      })
      .reduce(
        (acc: Array<Array<string>>, val: Array<string>) => {
          acc[0].push(val[0])
          acc[1].push(val[1])
          acc[2].push(val[2])

          return acc
        },
        [[], [], []],
      )

    const moveCallData = encodeMoveCallData(
      '0x3::session_key::create_session_key_with_multi_scope_entry',
      [],
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
        {
          type: { Vector: 'Address' },
          value: scopeModuleAddresss,
        },
        {
          type: { Vector: 'Ascii' },
          value: scopeModuleNames,
        },
        {
          type: { Vector: 'Ascii' },
          value: scopeFunctionNames,
        },
        {
          type: 'U64',
          value: BigInt(maxInactiveInterval),
        },
      ],
    )

    const params = [
      {
        from: account,
        to: '0xd46e8dd67c5d32be8058bb8eb970870f07244568', //TODO，can be fixed rooch address
        gas: '0x5F5E100', // 100000000
        value: '0x4e72a', // 2441406250
        data: hexlify(moveCallData),
      },
    ]

    const tx = await eth.sendTransaction(params)
    const result = await eth.waitTxConfirmed(tx)
    console.log(`result:`, result)
  }

  const requestWalletCreateSessionKey = async (
    provider: IClient,
    account: AccountDataType,
    scope: Array<string>,
    maxInactiveInterval: number,
  ): Promise<IAccount | null> => {
    const pk = Ed25519Keypair.generate()
    const authKey = pk.getPublicKey().toRoochAddress()

    try {
      await registerSessionKey(account.address, authKey, scope, maxInactiveInterval)

      const key = makeSessionAccountStoreKey(provider.getChainId(), account.roochAddress)
      window.sessionStorage.setItem(key, pk.export().privateKey)
      const authorizer = new PrivateKeyAuth(pk)

      return new Account(provider, account.roochAddress, authorizer)
    } catch (err: any) {
      console.log(`registerSessionKey error:`, err)

      const subStatus = parseRoochErrorSubStatus(err.message)
      if (subStatus) {
        throw new Error(
          'create session key fail, error category: ' +
            getErrorCategoryName(subStatus.category) +
            ', reason: ' +
            subStatus.reason,
        )
      }

      throw new Error('create session key error, reason:' + err.message)
    }
  }

  const requestPrivateCreateSessionKey = async (
    provider: IClient,
    account: IAccount,
    scope: Array<string>,
    maxInactiveInterval: number,
  ): Promise<IAccount | null> => {
    const pk = Ed25519Keypair.generate()
    const roochAddress = pk.getPublicKey().toRoochAddress()

    try {
      await account.registerSessionKey(roochAddress, scope, maxInactiveInterval)

      const key = makeSessionAccountStoreKey(provider.getChainId(), account.getAddress())
      window.sessionStorage.setItem(key, pk.export().privateKey)
      const authorizer = new PrivateKeyAuth(pk)

      return new Account(provider, account.getAddress(), authorizer)
    } catch (err: any) {
      console.log(`registerSessionKey error:`, err)

      const subStatus = parseRoochErrorSubStatus(err.message)
      if (subStatus) {
        throw new Error(
          'create session key fail, error category: ' +
            getErrorCategoryName(subStatus.category) +
            ', reason: ' +
            subStatus.reason,
        )
      }

      throw new Error('create session key error, reason:' + err.message)
    }
  }

  const requestAuthorize = async (scope: Array<string>, maxInactiveInterval: number) => {
    setLoading(true)

    try {
      const defaultAccount = auth.defaultAccount

      if (!defaultAccount) {
        setSessionAccount(null)

        return
      }

      if (defaultAccount.kp != null) {
        const roochAddress = defaultAccount.address
        const authorizer = new PrivateKeyAuth(defaultAccount.kp)
        const account = new Account(rooch.provider!, roochAddress, authorizer)

        const sessionAccount = await requestPrivateCreateSessionKey(
          filterdProvider,
          account,
          scope,
          maxInactiveInterval,
        )

        if (sessionAccount) {
          setSessionAccount(sessionAccount)
        }
      } else if (defaultAccount.type === 'ETH') {
        const sessionAccount = await requestWalletCreateSessionKey(
          filterdProvider,
          defaultAccount,
          scope,
          maxInactiveInterval,
        )

        if (sessionAccount) {
          setSessionAccount(sessionAccount)
        }
      }
    } catch (e: any) {
      setErrorMsg(e.message)
      setTimeout(() => {
        setErrorMsg(null)
      }, 5000)
    } finally {
      setLoading(false)
    }
  }

  const closeSession = () => {
    const defaultAccount = auth.defaultAccount
    if (defaultAccount && defaultAccount.type === AccountType.ROOCH) {
      clearSessionAccountInSessionStorage(rooch.provider!, defaultAccount.roochAddress)
    }
  }

  const getDefaultSession = (): string => {
    return ''

    // return window.sessionStorage.getItem(makeSessionAccountStoreKey(rooch.provider!.getChainId(), auth.defaultAccount?.roochAddress ?? '')) ?? ''
  }

  const session = {
    loading,
    initialization,
    account: sessionAccount,
    errorMsg,
    requestAuthorize,
    defaultSession: getDefaultSession(),
    close: closeSession,
  } as Session

  return <SessionContext.Provider value={session}>{children}</SessionContext.Provider>
}

export { SessionContext, SessionProvider }
