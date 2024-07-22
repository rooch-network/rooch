import { IDatasource } from '@sadoprotocol/ordit-sdk'

import { InscriptionID } from '../types'
import { inscriptionIDToString, fromB64 } from '../utils'
import { IGenerator, IGeneratorLoader } from './interface'
import { WasmGenerator } from './wasm_generator'

export class GeneratorLoader implements IGeneratorLoader {
  private datasource: IDatasource

  constructor(datasource: IDatasource) {
    this.datasource = datasource
  }

  public async load(inscription_id: InscriptionID): Promise<IGenerator> {
    const wasmInscription = await this.datasource.getInscription({
      id: inscriptionIDToString(inscription_id),
      decodeMetadata: false,
    })

    console.log('wasmInscription:', wasmInscription)

    const wasmBytes = fromB64(wasmInscription.mediaContent)
    return await WasmGenerator.loadWasmModule(wasmBytes)
  }
}
