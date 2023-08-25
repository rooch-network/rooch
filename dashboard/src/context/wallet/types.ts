export type AddChinaParameterType = {
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

export type MetamaskValueType = {
  loading: boolean
  hasProvider: boolean
  chainId: string|null,
  accounts: string[]
  isConnect: boolean
  connect: () => Promise<void>
  disconnect: () => void
  switchChina:(chainId: string) => Promise<void> 
  addChina: (params: AddChinaParameterType) => Promise<void>
}
