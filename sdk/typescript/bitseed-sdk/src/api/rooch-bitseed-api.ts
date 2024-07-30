import { SFTRecord } from '../types'
import { RoochBitSeedApiInterface } from './rooch-bitseed-api.interface'

export class RoochBitSeedApi implements RoochBitSeedApiInterface {
  getBitSeedSFTByID(): Promise<SFTRecord> {
    throw new Error('Method not implemented.')
  }
}
