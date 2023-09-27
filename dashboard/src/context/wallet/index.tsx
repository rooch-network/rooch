// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** ETH
import detectEthereumProvider from '@metamask/detect-provider'

// ** Types
import { ETHValueType } from 'src/context/wallet/types'

// ** SDK
import { ChainInfo, DevChain } from '@rooch/sdk'

type Props = {
  children: ReactNode
}

const defaultProvider: ETHValueType = {
  loading: true,
  chain: DevChain.info,
  hasProvider: false,
  provider: undefined,
  accounts: [],
  isConnect: false,
  switchChina: async () => Promise.resolve(),
  addChina: async () => Promise.resolve(),
  connect: async () => Promise.resolve(),
  disconnect: () => null,
}

const ETHContext = createContext(defaultProvider)

const ETHProvider = ({ children }: Props) => {
  const [hasProvider, setHasProvider] = useState<boolean>(defaultProvider.hasProvider)
  const [accounts, setAccounts] = useState<string[]>(defaultProvider.accounts)
  const [chain, setChainId] = useState<ChainInfo>(defaultProvider.chain)
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
      console.log('hash')
      console.log(Boolean(provider))
      console.log(hasProvider)

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

  console.log('aaa')
  console.log(hasProvider)

  const updateWallet = (accounts: any) => {
    setAccounts(accounts)
  }

  const connect = async (targetChain?: ChainInfo) => {
    let connectChain = targetChain ?? chain

    if (chain?.chainId !== connectChain.chainId) {
      try {
        await switchChina(connectChain)
      } catch (e: any) {
        console.log('connect error', e.toString())

        return
      }
    }

    return window.ethereum
      ?.request({
        method: 'eth_requestAccounts',
      })
      .then((accounts: any) => {
        updateWallet(accounts)
      })
  }

  const switchChina = async (chain: ChainInfo) => {
    try {
      await window.ethereum?.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: chain.chainId }],
      })
    } catch (e: any) {
      if (e.code === 4902) {
        // Rooch chain not found
        // try {
        await addChina(chain)

        // } catch (e: any) { // add china error
        //   console.log('eth switch chain err ', e.toString())
        //   return
        // }
      } else {
        // unknown error
        throw e
      }
    }

    setChainId(chain)
  }

  const addChina = async (chain: ChainInfo) => {
    return window.ethereum
      ?.request({
        method: 'wallet_addEthereumChain',
        params: [chain],
      })
      .then((v) => {
        console.log(v)
      })
  }

  const disconnect = async () => {
    if (window.ethereum?.isConnected()) {
      console.log(window.ethereum)
    }
  }

  const values = {
    loading,
    chain,
    hasProvider,
    provider: hasProvider ? window.ethereum : null,
    accounts,
    isConnect: hasProvider,
    addChina,
    switchChina,
    connect,
    disconnect,
  } as ETHValueType

  return <ETHContext.Provider value={values}>{children}</ETHContext.Provider>
}

export { ETHContext, ETHProvider }
