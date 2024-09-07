// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { debug } from 'debug'
import { IDatasource } from '@sadoprotocol/ordit-sdk'

import { InscriptionID } from '../types/index.js'
import { inscriptionIDToString, fromB64 } from '../utils/index.js'
import { IGenerator, IGeneratorLoader } from './interface.js'
import { WasmGenerator } from './wasm_generator.js'

const log = debug('bitseed:generator:loader')

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

    log('wasmInscription:', wasmInscription)

    const wasmBytes = fromB64(wasmInscription.mediaContent)
    return await WasmGenerator.loadWasmModule(wasmBytes)
  }
}
