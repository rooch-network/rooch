// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { createContext, useEffect, useState, ReactNode } from 'react'

// ** SDK
import { JsonRpcProvider, Chain, AllChain, DevChain } from '@rooch/sdk'

// ** Types
import { RoochProviderValueType } from 'src/context/rooch/types'

// ** Config
import authConfig from 'src/configs/auth'

// ** Hooks
import { useETH } from 'src/hooks/useETH'
import { ETHContext } from '../wallet'
import { ca } from 'date-fns/locale'

type Props = {
  children: ReactNode
}

const defaultProvider: RoochProviderValueType = {
  provider: null,
  switchChina: async () => Promise.resolve(),
  addChina: async () => Promise.resolve(),
  deleteChina: async () => Promise.resolve(),
  getAllChina: () => [],
  getActiveChina: () => DevChain,
}

const RoochContext = createContext(defaultProvider)

const RoochProvider = ({ children }: Props) => {
  // ** Hooks
  const eth = useETH()

  // ** States
  const [provider, setProvider] = useState<JsonRpcProvider | null>(defaultProvider.provider)
  const [china, setChina] = useState<Chain>(DevChain)

  useEffect(() => {
    const init = async (): Promise<void> => {
      const activeChainID =
        window.localStorage.getItem(authConfig.activeChain) ?? DevChain.id.toString()
      const chains = getAllChina()

      let chain = chains.find((v) => v.info.chainId === activeChainID)

      // default
      if (!chain) {
        chain = DevChain
      }

      console.log('set provider')
      console.log(chain)

      setProvider(new JsonRpcProvider(chain))
    }

    init()
  }, [])

  const getCustomChains = () => {
    let chainStr = window.localStorage.getItem(authConfig.chains)
    let chains: Chain[] = []

    if (chainStr) {
      chains = JSON.parse(chainStr)
    }

    return chains
  }

  const saveCustomChain = (chain: Chain) => {
    if (AllChain.some((v) => v.id === chain.id)) {
      return
    }

    let chains = getCustomChains()

    if (chains.some((v) => v.id === chain.id)) {
      return // chain already existed
    }

    chains.push(chain)

    window.localStorage.setItem(authConfig.chains, JSON.stringify(chains))
  }

  const deleteCustomChain = (chain: Chain) => {
    let chains = getCustomChains().filter((v) => v.id === chain.id)

    window.localStorage.setItem(authConfig.chains, JSON.stringify(chains))
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
    if (eth.isConnect) {
      await eth.switchChina(chain.info)
    }

    provider?.switchChain(chain)
    window.localStorage.setItem(authConfig.activeChain, chain.info.chainId)
  }

  const deleteChina = async (chain: Chain) => {
    deleteCustomChain(chain)

    // TODO: remove wallet chain
  }

  const getActiveChina = () => {
    const activeChinaID = parseInt(
      window.localStorage.getItem(authConfig.activeChain) ?? DevChain.id.toString(),
    )

    return getAllChina().find((v) => activeChinaID === v.id) ?? DevChain
  }

  const getAllChina = () => {
    return getCustomChains().concat(AllChain)
  }

  const values = {
    provider,
    addChina,
    switchChina,
    deleteChina,
    getAllChina,
    getActiveChina,
  }

  return <RoochContext.Provider value={values}> {children} </RoochContext.Provider>
}

export { RoochProvider, RoochContext }
