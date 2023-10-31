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

// ** Hooks
import { useRooch } from 'src/hooks/useRooch'

type Props = {
  children: ReactNode
}

const defaultProvider: ETHValueType = {
  loading: true,
  chainId: DevChain.info.chainId,
  hasProvider: false,
  provider: undefined,
  accounts: [],
  isConnect: false,
  sendTransaction: async () => Promise.resolve(),
  waitTxConfirmed: async () => Promise.resolve(),
  switchChina: async () => Promise.resolve(),
  addChina: async () => Promise.resolve(),
  connect: async () => Promise.resolve(),
  disconnect: () => null,
}

const ETHContext = createContext(defaultProvider)

const ETHProvider = ({ children }: Props) => {
  // Hooks
  const rooch = useRooch()

  // States
  const [hasProvider, setHasProvider] = useState<boolean>(defaultProvider.hasProvider)
  const [accounts, setAccounts] = useState<string[]>(defaultProvider.accounts)
  const [chainId, setChainId] = useState<string>(defaultProvider.chainId)
  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  useEffect(() => {
    setLoading(true)
    const refreshAccounts = (newAccounts: any) => {
      if (newAccounts && newAccounts.length > 0) {
        updateWallet(newAccounts)
      } else {
        updateWallet([])
      }
    }
    const refreshChina = (chainId: any) => {
      setChainId(chainId)

      // TODO: handle switch to unknown chain ?
      rooch.switchByChinaId(chainId)
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
  }, [rooch])

  const updateWallet = (accounts: any) => {
    setAccounts(accounts)
  }

  const connect = async (targetChain?: ChainInfo) => {
    let connectChain = targetChain ?? rooch.getActiveChina().info

    if (chainId !== connectChain.chainId) {
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

        return accounts
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
        await addChina(chain)
      } else {
        // unknown error
        console.log(e)
        throw e
      }
    }

    setChainId(chain.chainId)
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
    // if (window.ethereum?.isConnected()) {
    //   console.log(window.ethereum)
    // }
  }

  const sendTransaction = async (params: unknown[]) => {
    const curChain = rooch.getActiveChina()

    console.log('cur chain ', curChain)

    if (String(curChain.id) !== chainId) {
      await switchChina(curChain.info)
    }

    return await window.ethereum?.request({
      method: 'eth_sendTransaction',
      params,
    })
  }

  const waitTxConfirmed = async (txHash: string) => {
    let receipt
    while (!receipt) {
      receipt = await window.ethereum?.request({
        method: 'eth_getTransactionReceipt',
        params: [txHash],
      })

      if (!receipt) {
        await new Promise((resolve) => setTimeout(resolve, 3000)) // wait for 3 seconds before checking again
      }
    }

    return receipt
  }

  const values = {
    loading,
    chainId,
    hasProvider,
    provider: hasProvider ? window.ethereum : null,
    accounts,
    isConnect: hasProvider,
    sendTransaction,
    waitTxConfirmed,
    addChina,
    switchChina,
    connect,
    disconnect,
  } as ETHValueType

  return <ETHContext.Provider value={values}>{children}</ETHContext.Provider>
}

export { ETHContext, ETHProvider }
