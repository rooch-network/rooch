// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'
import detectEthereumProvider from '@metamask/detect-provider'
import MetaMaskSDK from '@metamask/sdk'

import { MetamaskValueType, AddChinaParameterType } from 'src/context/wallet/types'

// ** Config
import config from 'src/configs/auth'

const options = {
  dappMetadata: { url: 'https://rooch.network', name: 'rooch dashboard' }
}

type Props = {
  children: ReactNode
}

const defaultProvider: MetamaskValueType = {
  loading: true,
  chainId: null,
  hasProvider: false,
  accounts: [],
  isConnect: false,
  switchChina: async () => {},
  addChina: async () => {},
  connect: async () => {},
  disconnect: () => null
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
      if (newAccounts.length > 0) {
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
    if (chainId !== config.roochChain.chainId) {
      try {
        await switchChina(config.roochChain.chainId)
      } catch (e: any) {
        if (e.code === 4902) {
          try {
            await addChina({
              ...config.roochChain
            })
          } catch (e) {
            return
          }
        } else {
          return 
        }
      }
    }

    return window.ethereum
      ?.request({
        method: 'eth_requestAccounts'
      })
      .then(accounts => {
        updateWallet(accounts)
      })
  }

  const switchChina = async (chainId: string) => {
    return window.ethereum
      ?.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: chainId }]
      })
      .then(value => {
        setChainId(chainId)
        console.log('switch success ' + value)
      })
  }

  const addChina = async (params: AddChinaParameterType) => {
    return window
      .ethereum!.request({
        method: 'wallet_addEthereumChain',
        params: params
      })
      .then(value => {
        console.log(value)
      })
  }

  const disconnect = () => {
    if (window.ethereum!.isConnected()) {
      console.log(window.ethereum)
      window.sdkProvider.handleDisconnect({ terminate: true })
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
    disconnect
  }

  return <MetamaskContext.Provider value={vlaues}>{children}</MetamaskContext.Provider>
}

export { MetamaskContext, MetamaskProvider }
