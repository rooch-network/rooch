import { SFTRecord } from '../types/index.js'
import { RoochBitSeedApiInterface } from './rooch-bitseed-api.interface.js'

export class RoochBitSeedApi implements RoochBitSeedApiInterface {
  getBitSeedSFTByID(): Promise<SFTRecord> {
    throw new Error('Method not implemented.')
  }
}
