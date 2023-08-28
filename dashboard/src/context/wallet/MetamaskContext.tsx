// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { createContext, useEffect, useState, ReactNode } from 'react'
import detectEthereumProvider from '@metamask/detect-provider'

import { MetamaskValueType, AddChinaParameterType } from 'src/context/wallet/types'

// ** Config
import config from 'src/configs/auth'

type Props = {
  children: ReactNode
}

const defaultProvider: MetamaskValueType = {
  loading: true,
  chainId: null,
  hasProvider: false,
  accounts: [],
  isConnect: false,
  switchChina: async () => Promise.resolve(),
  addChina: async () => Promise.resolve(),
  connect: async () => Promise.resolve(),
  disconnect: () => null,
}

const MetamaskContext = createContext(defaultProvider)

const MetamaskProvider = ({ children }: Props) => {
  const [hasProvider, setHasProvider] = useState<boolean>(defaultProvider.hasProvider)
  const [accounts, setAccounts] = useState<string[]>(defaultProvider.accounts)
  const [chainId, setChainId] = useState<string | null>(defaultProvider.chainId)
  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  useEffect(() => {
    setLoading(true)

    const refreshAccounts = (newAccounts: any) => {
      console.log(newAccounts)
      if (newAccounts && newAccounts.length > 0) {
        updateWallet(newAccounts)
      } else {
        updateWallet([])
      }
    }

    const refreshChina = (chainId: any) => {
      setChainId(chainId)
    }

    const getProvider = async () => {
      const provider = await detectEthereumProvider({ silent: true })
      setHasProvider(Boolean(provider))

      if (provider) {
        const chainId = await window.ethereum?.request({ method: 'eth_chainId' })
        refreshChina(chainId)

        const accounts = await window.ethereum?.request({ method: 'eth_accounts' })
        refreshAccounts(accounts)

        window.ethereum?.on('chainChanged', refreshChina)
        window.ethereum?.on('accountsChanged', refreshAccounts)
      }
    }

    getProvider().finally(() => {
      setLoading(false)
    })

    return () => {
      window.ethereum?.removeListener('chainChanged', refreshChina)
      window.ethereum?.removeListener('accountsChanged', refreshAccounts)
    }
  }, [])

  const updateWallet = (accounts: any) => {
    setAccounts(accounts)
  }

  const connect = async () => {
    console.log('开始链接')
    if (chainId !== config.roochChain.chainId) {
      try {
        await switchChina(config.roochChain.chainId)
      } catch (e: any) {
        if (e.code === 4902) {
          // Rooch chain not found
          try {
            await addChina({
              ...config.roochChain,
            })
          } catch (e) {
            return
          }
        } else {
          return
        }
      }
    }

    console.log('开始链接')

    return window.ethereum
      ?.request({
        method: 'eth_requestAccounts',
      })
      .then((accounts: any) => {
        updateWallet(accounts)
      })
  }

  const switchChina = async (chainId: string) => {
    return window.ethereum
      ?.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: chainId }],
      })
      .then((value: any) => {
        setChainId(chainId)
        console.log('switch success ' + value)
      })
      .catch((e) => {
        console.log(e)
      })
  }

  const addChina = async (params: AddChinaParameterType) => {
    return window.ethereum
      ?.request({
        method: 'wallet_addEthereumChain',
        params: params,
      })
      .then((v) => {
        console.log(v)
      })
  }

  const disconnect = () => {
    if (window.ethereum!.isConnected()) {
      console.log(window.ethereum)
    }
  }

  const vlaues = {
    loading,
    chainId,
    hasProvider,
    accounts,
    isConnect: hasProvider && Boolean(window.ethereum?.isConnected()),
    addChina,
    switchChina,
    connect,
    disconnect,
  }

  return <MetamaskContext.Provider value={vlaues}>{children}</MetamaskContext.Provider>
}

export { MetamaskContext, MetamaskProvider }
