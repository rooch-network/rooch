import { SFTRecord } from '../types/index.js'

export interface RoochBitSeedApiInterface {
  getBitSeedSFTByID(): Promise<SFTRecord>
}
