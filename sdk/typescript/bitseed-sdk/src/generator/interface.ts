import { InscriptionID, SFTRecord } from '../types/index.js'
import { InscribeSeed } from './seed.js'
export interface IGenerator {
  inscribeGenerate(deployArgs: Array<string>, seed: InscribeSeed, userInput: string): Promise<SFTRecord>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
