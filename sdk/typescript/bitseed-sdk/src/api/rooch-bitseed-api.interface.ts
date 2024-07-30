import { SFTRecord } from '../types'

export interface RoochBitSeedApiInterface {
  getBitSeedSFTByID(): Promise<SFTRecord>
}
