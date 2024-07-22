import { InscriptionID, SFTRecord } from '../types'
import { InscribeSeed } from './seed'
export interface IGenerator {
  inscribeGenerate(deployArgs: Array<string>, seed: InscribeSeed, userInput: string): Promise<SFTRecord>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
