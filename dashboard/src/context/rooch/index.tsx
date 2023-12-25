// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** SDK
import { RoochClient, Chain, AllChain, DevChain } from '@roochnetwork/rooch-sdk'

// ** Types
import { RoochProviderValueType } from 'src/context/rooch/types'

// ** Config
import authConfig from 'src/configs/auth'

type Props = {
  children: ReactNode
}

const defaultProvider: RoochProviderValueType = {
  provider: null,
  loading: true,
  switchChain: async () => Promise.resolve(),
  switchByChainId: async () => Promise.resolve(),
  addChain: async () => Promise.resolve(),
  deleteChain: async () => Promise.resolve(),
  getAllChain: () => [],
  getActiveChain: () => DevChain,
}

const RoochContext = createContext(defaultProvider)

const RoochProvider = ({ children }: Props) => {
  // ** States
  const [provider, setProvider] = useState<RoochClient | null>(defaultProvider.provider)

  const [loading, setLoading] = useState<boolean>(defaultProvider.loading)

  useEffect(() => {
    const init = async (): Promise<void> => {
      const activeChainID =
        window.localStorage.getItem(authConfig.activeChain) ?? DevChain.info.chainId

      let chainStr = window.localStorage.getItem(authConfig.chains)
      let chains = AllChain

      if (chainStr) {
        chains = chains.concat(
          JSON.parse(chainStr).map(
            (v: any) =>
              new Chain(v.id, v.name, {
                ...v.options,
              }),
          ),
        )
      }

      let chain = chains.find((v) => v.info.chainId === activeChainID)

      // default
      if (!chain) {
        chain = DevChain
      }

      setProvider(new RoochClient(chain))
    }

    init().finally(() => setLoading(false))
  }, [])

  const getCustomChains = () => {
    let chainStr = window.localStorage.getItem(authConfig.chains)
    let chains: Chain[] = []

    if (chainStr) {
      chains = JSON.parse(chainStr).map(
        (v: any) =>
          new Chain(v.id, v.name, {
            ...v.options,
          }),
      )
    }

    return chains
  }

  const saveCustomChain = (chain: Chain) => {
    let chains = getCustomChains()

    if (chains.some((v) => v.id === chain.id && v.url === chain.url)) {
      console.info('chain already existed')

      return
    }

    chains.push(chain)

    window.localStorage.setItem(authConfig.chains, JSON.stringify(chains))
  }

  const deleteCustomChain = (chain: Chain) => {
    let chains = getCustomChains().filter((v) => v.id === chain.id)

    window.localStorage.setItem(authConfig.chains, JSON.stringify(chains))
  }

  const getAllChain = () => {
    return getCustomChains().concat(AllChain)
  }

  const addChain = async (chain: Chain) => {
    try {
      await switchChain(chain)
    } catch (e) {
      return
    }

    saveCustomChain(chain)
  }

  const switchChain = async (chain: Chain) => {
    provider?.switchChain(chain)
    window.localStorage.setItem(authConfig.activeChain, chain.info.chainId)
  }

  const switchByChainId = async (chainId: string) => {
    const chain = getAllChain().find((v) => v.info.chainId === chainId)

    if (!chain || !provider) {
      return
    }

    if (provider?.chain.info.chainId === chainId) {
      return
    }

    await switchChain(chain)
    window.location.reload()
  }

  const deleteChain = async (chain: Chain) => {
    deleteCustomChain(chain)

    // TODO: remove wallet chain
  }

  const getActiveChain = () => {
    const activeChainID = parseInt(
      window.localStorage.getItem(authConfig.activeChain) ?? DevChain.id.toString(),
    )

    return getAllChain().find((v) => activeChainID === v.id) ?? DevChain
  }

  const values = {
    provider,
    loading,
    addChain,
    switchChain,
    switchByChainId,
    deleteChain,
    getAllChain,
    getActiveChain,
  }

  return <RoochContext.Provider value={values}> {children} </RoochContext.Provider>
}

export { RoochProvider, RoochContext }
