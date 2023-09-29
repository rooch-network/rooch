// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { hexlify } from '@ethersproject/bytes'
import { useState } from 'react'
import { useAuth } from 'src/hooks/useAuth'
import { useRooch } from 'src/hooks/useRooch'

import { AccountDataType } from 'src/context/auth/types'

// ** Rooch SDK
import {
  bcsTypes,
  IAccount,
  Account,
  PrivateKeyAuth,
  Ed25519Keypair,
  encodeMoveCallData,
  addressToSeqNumber,
} from '@rooch/sdk'

// ** React Imports
import { createContext, ReactNode } from 'react'
import { Session } from 'src/context/session/types'

type Props = {
  children: ReactNode
}

const SessionContext = createContext<Session>({
  loading: false,
  account: undefined,
  requestAuthorize: undefined,
})

const makeSessionAccountStoreKey = (address: string) => {
  return `rooch::dashboard::account::${address}::current-session-key`
}

const SessionProvider = ({ children }: Props) => {
  const auth = useAuth()
  const rooch = useRooch()

  const [loading, setLoading] = useState(false)

  const [sessionAccount, setSessionAccount] = useState<IAccount | undefined>(() => {
    const defaultAccount = auth.defaultAccount()

    if (defaultAccount) {
      try {
        // Get from local storage by key
        const secretKey = window.localStorage.getItem(
          makeSessionAccountStoreKey(defaultAccount.address),
        )

        if (secretKey) {
          let sk = bcsTypes.fromB64(secretKey)

          // The rooch cli generated key contains schema, remove it
          if (sk.length > 32) {
            sk = sk.slice(1)
          }

          const pk = Ed25519Keypair.fromSecretKey(sk)
          const authorizer = new PrivateKeyAuth(pk)

          return new Account(rooch.provider!, defaultAccount.roochAddress, authorizer)
        }
      } catch (error) {
        // If error also return initialValue
        console.log(error)
      }
    }

    return undefined
  })

  const waitTxConfirmed = async (ethereum: any, txHash: string) => {
    let receipt
    while (!receipt) {
      receipt = await ethereum.request({
        method: 'eth_getTransactionReceipt',
        params: [txHash],
      })

      if (!receipt) {
        await new Promise((resolve) => setTimeout(resolve, 3000)) // wait for 3 seconds before checking again
      }
    }

    return receipt
  }

  const registerSessionKey = async (
    ethereum: any,
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

    const tx = await ethereum.request({
      method: 'eth_sendTransaction',
      params,
    })

    const result = await waitTxConfirmed(ethereum, tx)
    console.log(`result:`, result)
  }

  const requestWalletCreateSessionKey = async (
    account: AccountDataType,
    scope: Array<string>,
    maxInactiveInterval: number,
  ): Promise<IAccount | null> => {
    const pk = Ed25519Keypair.generate()
    const authKey = pk.getPublicKey().toRoochAddress()

    try {
      await registerSessionKey(
        window.ethereum,
        account.address,
        authKey,
        scope,
        maxInactiveInterval,
      )

      const key = makeSessionAccountStoreKey(account.address)
      window.localStorage.setItem(key, pk.export().privateKey)
      const authorizer = new PrivateKeyAuth(pk)

      return new Account(rooch.provider!, account.roochAddress, authorizer)
    } catch (err: any) {
      console.log(`registerSessionKey error:`, err)

      return null
    }
  }

  const requestPrivateCreateSessionKey = async (
    account: IAccount,
    scope: Array<string>,
    maxInactiveInterval: number,
  ): Promise<IAccount | null> => {
    const pk = Ed25519Keypair.generate()
    const roochAddress = pk.getPublicKey().toRoochAddress()

    try {
      await account.registerSessionKey(roochAddress, scope, maxInactiveInterval)

      const key = makeSessionAccountStoreKey(account.getAddress())
      window.localStorage.setItem(key, pk.export().privateKey)
      const authorizer = new PrivateKeyAuth(pk)

      return new Account(rooch.provider!, roochAddress, authorizer)
    } catch (err: any) {
      console.log(`registerSessionKey error:`, err)

      return null
    }
  }

  const requestAuthorize = async (scope: Array<string>, maxInactiveInterval: number) => {
    setLoading(true)

    try {
      const defaultAccount = auth.defaultAccount()
      if (!defaultAccount) {
        setSessionAccount(undefined)

        return
      }

      if (defaultAccount != null) {
        if (defaultAccount.kp != null) {
          const roochAddress = defaultAccount.address
          const authorizer = new PrivateKeyAuth(defaultAccount.kp)
          const account = new Account(rooch.provider!, roochAddress, authorizer)

          const sessionAccount = await requestPrivateCreateSessionKey(
            account,
            scope,
            maxInactiveInterval,
          )

          if (sessionAccount) {
            setSessionAccount(sessionAccount)
          }
        } else if (defaultAccount.type === 'ETH') {
          const sessionAccount = await requestWalletCreateSessionKey(
            defaultAccount,
            scope,
            maxInactiveInterval,
          )

          if (sessionAccount) {
            setSessionAccount(sessionAccount)
          }
        }
      }
    } finally {
      setLoading(false)
    }
  }

  const session = {
    loading,
    account: sessionAccount,
    requestAuthorize,
  }

  return <SessionContext.Provider value={session}>{children}</SessionContext.Provider>
}

export { SessionContext, SessionProvider }
