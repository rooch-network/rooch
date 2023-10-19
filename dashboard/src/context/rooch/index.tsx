// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** SDK
import {JsonRpcProvider, Chain, AllChain, DevChain, LocalChain} from '@rooch/sdk'

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
  switchChina: async () => Promise.resolve(),
  switchByChinaId: async () => Promise.resolve(),
  addChina: async () => Promise.resolve(),
  deleteChina: async () => Promise.resolve(),
  getAllChina: () => [],
  getActiveChina: () => DevChain,
}

const RoochContext = createContext(defaultProvider)

const RoochProvider = ({ children }: Props) => {
  // ** States
  const [provider, setProvider] = useState<JsonRpcProvider | null>(defaultProvider.provider)

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

      console.log('设置 provider')
      setProvider(new JsonRpcProvider(chain))
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
    // if (AllChain.some((v) => v.id === chain.id)) {
    //   return
    // }

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

  const getAllChina = () => {
    return getCustomChains().concat(AllChain)
  }

  const addChina = async (chain: Chain) => {
    try {
      await switchChina(chain)
    } catch (e) {
      return
    }

    saveCustomChain(chain)
  }

  const switchChina = async (chain: Chain) => {
    provider?.switchChain(chain)
    window.localStorage.setItem(authConfig.activeChain, chain.info.chainId)
  }

  const switchByChinaId = async (chainId: string) => {
    const chain = getAllChina().find((v) => v.info.chainId === chainId)

    if (!chain || !provider) {
      return
    }

    if (provider?.chain.info.chainId === chainId) {
      return
    }

    await switchChina(chain)
    window.location.reload()
  }

  const deleteChina = async (chain: Chain) => {
    deleteCustomChain(chain)

    // TODO: remove wallet chain
  }

  const getActiveChina = () => {
    const activeChinaID = parseInt(
      window.localStorage.getItem(authConfig.activeChain) ?? LocalChain.id.toString(),
    )

    return getAllChina().find((v) => activeChinaID === v.id) ?? DevChain
  }

  const values = {
    provider,
    loading,
    addChina,
    switchChina,
    switchByChinaId,
    deleteChina,
    getAllChina,
    getActiveChina,
  }

  return <RoochContext.Provider value={values}> {children} </RoochContext.Provider>
}

export { RoochProvider, RoochContext }
