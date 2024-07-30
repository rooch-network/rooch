import { 
  SatPoint, 
  FeeRate, 
  Amount, 
  Generator, 
  InscriptionID 
} from '../types'

export interface NestedObject {
  [key: string]: NestedObject | any
}

// InscribeOptions interface
export interface InscribeOptions {
  /**
   * Inscribe <SATPOINT>. This SatPoint will be used as mint seed.
   */
  satpoint?: SatPoint

  /**
   * Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.
   * Defaults to <FEE_RATE> if unset.
   */
  commit_fee_rate?: FeeRate

  /**
   * Send inscription to <DESTINATION>.
   */
  destination?: string

  /**
   * Don't sign or broadcast transactions.
   */
  dry_run?: boolean

  /**
   * Use fee rate of <FEE_RATE> sats/vB.
   */
  fee_rate: FeeRate

  /**
   * Amount of postage to include in the inscription. Default `10000sat`.
   */
  postage?: Amount

  /**
   * meta for Inscribe
   */
  meta?: NestedObject
}

export interface DeployOptions extends InscribeOptions {
  repeat?: number
  deploy_args?: Array<string>
}

export interface APIInterface {
  generator(name: string, wasmBytes: Uint8Array, opts?: InscribeOptions): Promise<InscriptionID>
  deploy(
    tick: string,
    max: number,
    generator: Generator,
    opts?: DeployOptions,
  ): Promise<InscriptionID>
  mint(
    tickInscriptionId: InscriptionID,
    userInput: string,
    opts?: InscribeOptions,
  ): Promise<InscriptionID>
  merge(a: InscriptionID, b: InscriptionID): Promise<InscriptionID>
  split(a: InscriptionID): Promise<[InscriptionID, InscriptionID]>
}
