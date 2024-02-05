// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

const LocalNetworkURL = 'http://127.0.0.1:50051'
const DevNetworkURL = 'https://dev-seed.rooch.network:443'
const TestNetworkURL = 'https://test-seed.rooch.network'

const DEV_NETWORK_ID = 0x134afd7
const LOCAL_NETWORK_ID = 0x134afd8
const TEST_NETWORK_ID = 0x134afd6
// const MAIN_NETWORK_ID = 0x134afd5

export interface ChainInfo {
  chainId: string
  blockExplorerUrls?: string[]
  chainName?: string
  iconUrls?: string[]
  nativeCurrency?: {
    name: string
    symbol: string
    decimals: number
  }
  rpcUrls?: string[]
}

interface ConnectionOptions {
  url: string
  websocket?: string
}

export class Network {
  id: number
  name: string
  options: ConnectionOptions

  constructor(id: number, name: string, options: ConnectionOptions) {
    this.id = id
    this.name = name
    this.options = options
  }

  get url() {
    return this.options.url
  }

  get websocket() {
    return this.options.websocket || this.options.url
  }

  get info(): ChainInfo {
    return {
      chainId: `0x${this.id.toString(16)}`,
      chainName: this.name,
      iconUrls: [
        'https://github.com/rooch-network/rooch/blob/main/docs/website/public/logo/rooch_black_text.png',
      ],
      nativeCurrency: {
        name: 'ROH',
        symbol: 'ROH',
        decimals: 18,
      },
      rpcUrls: [this.options.url],
    }
  }
}

export const LocalNetwork = new Network(LOCAL_NETWORK_ID, 'local', {
  url: LocalNetworkURL,
})

export const DevNetwork = new Network(DEV_NETWORK_ID, 'dev', {
  url: DevNetworkURL,
})

export const TESTNetwork = new Network(TEST_NETWORK_ID, 'dev', {
  url: TestNetworkURL,
})

export const AllNetwork = [LocalNetwork, DevNetwork, TESTNetwork]
