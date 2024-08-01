
export type OutPoint = {
  txid: string
  vout: number
}

export type SatPoint = {
  outpoint: OutPoint
  offset: number
}

export type FeeRate = number
export type Amount = number