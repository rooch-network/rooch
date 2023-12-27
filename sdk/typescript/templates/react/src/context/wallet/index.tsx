// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** ETH
import detectEthereumProvider from '@metamask/detect-provider'

// ** Types
import { ETHValueType } from './types'
import { AccountDataType, AccountType } from '../types'
import { RoochProviderValueType } from '../rooch/types'

// ** SDK
import { ChainInfo, DevChain } from '@roochnetwork/rooch-sdk'

// ** config
import config from '../../config/index'

// ** Hooks
import { useRooch } from '@/hooks/useRooch'

type Props = {
  children: ReactNode
}

const defaultProvider: ETHValueType = {
  loading: true,
  chainId: DevChain.info.chainId,
  hasProvider: false,
  provider: undefined,
  accounts: new Map(),
  isConnect: false,
  activeAccount: null,
  sendTransaction: async () => Promise.resolve(),
  waitTxConfirmed: async () => Promise.resolve(),
  switchChain: async () => Promise.resolve(),
  addChain: async () => Promise.resolve(),
  connect: async () => Promise.resolve(),
  disconnect: () => null,
}

const ETHContext = createContext(defaultProvider)

const ETHProvider = ({ children }: Props) => {
  // Hooks
  const rooch = useRooch()

  // States
  const [hasProvider, setHasProvider] = useState<boolean>(defaultProvider.hasProvider)
  const [accounts, setAccounts] = useState<Map<string, AccountDataType>>(defaultProvider.accounts)
  const [activeAccount, setActiveAccount] = useState(defaultProvider.activeAccount)
  const [roochAddressMap, setRoochAddressMap] = useState<Map<string, string>>(new Map())
  const [chainId, setChainId] = useState<string>(defaultProvider.chainId)
  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  useEffect(() => {
    const roochAddressMapStr = window.localStorage.getItem(config.roochAccountMap)

    if (roochAddressMapStr) {
      setRoochAddressMap(new Map<string, string>(JSON.parse(roochAddressMapStr)))
    }
  }, [])

  useEffect(() => {
    if (!rooch) {
      return
    }

    setLoading(true)

    const refreshAccounts = (newAccounts: any) => {
      updateWallet(rooch, roochAddressMap, newAccounts)
    }
    const refreshChain = (chainId: any) => {
      setChainId(chainId)

      // TODO: handle switch to unknown chain ?
      rooch.switchByChainId(chainId)
    }

    const updateWallet = async (
      rooch: RoochProviderValueType,
      roochAddressMap: Map<string, string>,
      ethAddresses: string[],
    ) => {
      if (ethAddresses.length === 0) {
        setAccounts(new Map())

        return
      }

      for (const ethAddress of ethAddresses) {
        let acc = accounts.get(ethAddress)
        if (acc && acc.roochAddress) {
          continue
        }

        let roochAddress = roochAddressMap.get(ethAddress)

        // Check whether roochAddress exists in the cache
        if (!roochAddress) {
          try {
            roochAddress = await rooch.provider?.resoleRoochAddress(ethAddress)!

            roochAddressMap.set(ethAddress, roochAddress!)
          } catch (e) {
            // Normally there should be no errors here，If it does, it must be a contract error
            console.log(
              'resole rooch address error, Please feedback here https://github.com/rooch-network/rooch',
            )
          }
        }

        accounts.set(ethAddress, {
          address: ethAddress,
          roochAddress: roochAddress!,
          activate: true,
          kp: null,
          type: AccountType.ETH,
        })
      }

      setActiveAccount(accounts.get(ethAddresses[0])!)

      window.localStorage.setItem(
        config.roochAccountMap,
        JSON.stringify(Array.from(roochAddressMap.entries())),
      )

      setAccounts(new Map([...accounts]))
    }

    const getProvider = async () => {
      const provider = await detectEthereumProvider({ silent: true })
      setHasProvider(Boolean(provider))

      if (provider) {
        const chainId = await window.ethereum?.request({ method: 'eth_chainId' })
        refreshChain(chainId)

        const accounts = await window.ethereum?.request({ method: 'eth_accounts' })
        refreshAccounts(accounts)

        window.ethereum?.on('chainChanged', refreshChain)
        window.ethereum?.on('accountsChanged', refreshAccounts)
      }
    }

    getProvider().finally(() => {
      setLoading(false)
    })

    return () => {
      window.ethereum?.removeListener('chainChanged', refreshChain)
      window.ethereum?.removeListener('accountsChanged', refreshAccounts)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rooch])

  const connect = async (targetChain?: ChainInfo) => {
    let connectChain = targetChain ?? rooch.getActiveChain().info

    if (chainId !== connectChain.chainId) {
      try {
        await switchChain(connectChain)
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
        return accounts
      })
  }

  const switchChain = async (chain: ChainInfo) => {
    try {
      await window.ethereum?.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: chain.chainId }],
      })
    } catch (e: any) {
      if (e.code === 4902) {
        await addChain(chain)
      } else {
        // unknown error
        console.log(e)
        throw e
      }
    }

    setChainId(chain.chainId)
  }

  const addChain = async (chain: ChainInfo) => {
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
    const curChain = rooch.getActiveChain()

    if (String(curChain.id) !== chainId) {
      await switchChain(curChain.info)
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
    activeAccount,
    isConnect: hasProvider ? window.ethereum?.isConnected() : false,
    sendTransaction,
    waitTxConfirmed,
    addChain,
    switchChain,
    connect,
    disconnect,
  } as ETHValueType

  return <ETHContext.Provider value={values}>{children}</ETHContext.Provider>
}

export { ETHContext, ETHProvider }
