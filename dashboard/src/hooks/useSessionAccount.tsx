// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react'
import { useAuth } from 'src/hooks/useAuth'
import { useMetamask } from 'src/hooks/useMetamask'
import { AccountDataType } from 'src/context/auth/types'

// ** Rooch SDK
import {
  IAccount,
  Account,
  JsonRpcProvider,
  PrivateKeyAuth,
  Ed25519Keypair,
  encodeMoveCallData,
  addressToSeqNumber,
} from '@rooch/sdk'

export default function useSessionAccount() {
  const auth = useAuth()
  const metaMask = useMetamask()
  const [loading, setLoading] = useState(false)
  const [sessionAccount, setSessionAccount] = useState<IAccount | undefined>(undefined)

  const registerSessionKey = async (
    ethereum: any,
    account: string,
    authKey: string,
    scope: string,
    maxInactiveInterval: number,
  ) => {
    const parts = scope.split('::')
    if (parts.length !== 3) {
      throw new Error('invalid scope')
    }

    const scopeModuleAddress = parts[0]
    const scopeModuleName = parts[1]
    const scopeFunctionName = parts[2]

    const moveCallData = encodeMoveCallData(
      '0x3::session_key::create_session_key_entry',
      [],
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
        {
          type: 'Address',
          value: scopeModuleAddress,
        },
        {
          type: 'Ascii',
          value: scopeModuleName,
        },
        {
          type: 'Ascii',
          value: scopeFunctionName,
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
        gas: '0x76c0', // 30400
        gasPrice: '0x9184e72a000', // 10000000000000
        value: '0x4e72a', // 2441406250
        data: moveCallData,
      },
    ]

    const tx = await ethereum.request({
      method: 'eth_sendTransaction',
      params,
    })
    console.log(`tx:`, tx)
  }

  const requestWalletCreateSessionKey = async (
    account: AccountDataType,
    scope: Array<string>,
    maxInactiveInterval: number,
  ): Promise<IAccount> => {
    const provider = new JsonRpcProvider()

    const pk = Ed25519Keypair.generate()
    const roochAddress = pk.toRoochAddress()

    await registerSessionKey(
      metaMask.provider,
      account.address,
      roochAddress,
      scope[0],
      maxInactiveInterval,
    )

    const authorizer = new PrivateKeyAuth(pk)

    return new Account(provider, roochAddress, authorizer)
  }

  const requestAuthorize = async (scope: Array<string>, maxInactiveInterval: number) => {
    setLoading(true)

    const defaultAccount = auth.defaultAccount()
    if (!defaultAccount) {
      setSessionAccount(undefined)

      return
    }

    if (defaultAccount != null) {
      if (defaultAccount.kp != null) {
        const provider = new JsonRpcProvider()

        const roochAddress = defaultAccount.address
        const authorizer = new PrivateKeyAuth(defaultAccount.kp)

        const account = new Account(provider, roochAddress, authorizer)
        const sessionAccount = await account.createSessionAccount(
          scope[0],
          60 * 20,
          maxInactiveInterval,
        )
        setSessionAccount(sessionAccount)
      } else if (defaultAccount.type === 'ETH') {
        const sessionAccount = await requestWalletCreateSessionKey(
          defaultAccount,
          scope,
          maxInactiveInterval,
        )
        setSessionAccount(sessionAccount)
      }
    }

    setLoading(false)
  }

  return { loading, sessionAccount, requestAuthorize }
}
