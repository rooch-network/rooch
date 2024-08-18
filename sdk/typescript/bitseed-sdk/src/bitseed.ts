// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { debug } from 'debug'
import * as bitcoin from 'bitcoinjs-lib'
import { IDatasource } from '@sadoprotocol/ordit-sdk'
import { Inscriber, Ordit, ordit, UTXOLimited } from '@sadoprotocol/ordit-sdk'

import { BITSEED_PROTOAL_NAME } from './constants/index.js'
import { InscriptionID, Generator, Tick, SFTRecord } from './types/index.js'
import {
  inscriptionIDToString,
  extractInscriptionID,
  toB64,
  decodeUTXOs,
  sleep,
} from './utils/index.js'
import { IGeneratorLoader, InscribeSeed } from './generator/index.js'
import { APIInterface, DeployOptions, InscribeOptions } from './interfaces/index.js'
import { BitseedSDKError } from './errors/index.js'

const log = debug('bitseed:bitseed')

export class BitSeed implements APIInterface {
  private network: bitcoin.Network
  private primaryWallet: Ordit
  private fundingWallet: Ordit
  private datasource: IDatasource
  private generatorLoader: IGeneratorLoader

  constructor(
    primaryWallet: Ordit,
    fundingWallet: Ordit,
    datasource: IDatasource,
    generatorLoader: IGeneratorLoader,
  ) {
    this.primaryWallet = primaryWallet
    this.fundingWallet = fundingWallet
    this.datasource = datasource
    this.generatorLoader = generatorLoader
    this.network = this.resolveNetwork(this.primaryWallet.network)
  }

  resolveNetwork(networkType: string): bitcoin.Network {
    if (networkType === 'regtest') {
      return bitcoin.networks.regtest
    } else if (networkType === 'testnet') {
      return bitcoin.networks.testnet
    } else {
      return bitcoin.networks.bitcoin
    }
  }

  public async inscribe(sft: SFTRecord, opts?: InscribeOptions): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    let meta = {
      op: sft.op,
      tick: sft.tick,
      amount: sft.amount,
      attributes: sft.attributes,
    }

    let contentType: string | undefined = undefined
    let body: string | undefined = undefined

    if (sft.content && sft.content.content_type && sft.content.body) {
      contentType = sft.content.content_type
      body = toB64(sft.content.body)
    }

    const inscriber = new Inscriber({
      datasource: this.datasource,
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: body,
      mediaType: contentType,
      feeRate: opts?.fee_rate || 1,
      meta: meta,
      postage: opts?.postage || 600, // base value of the inscription in sats
    })

    inscriber.withMetaProtocol(BITSEED_PROTOAL_NAME)

    const revealed = await inscriber.generateCommit()
    log('revealed:', revealed)

    // deposit revealFee to address
    const utxos = await this.depositRevealFee(revealed, opts)
    log('depositRevealFee utxos:', utxos)

    let ready = false

    try {
      const setUTXOs = (builder: any, utxos: UTXOLimited[]) => {
        builder.utxos = utxos
        builder.suitableUnspent = utxos[0]
        builder.ready = true
      }

      setUTXOs(inscriber, utxos)
      ready = true
    } catch (error) {
      log('inscribe error:', error)
      ready = false
    }

    if (ready) {
      await inscriber.build()

      const signedTxHex = this.primaryWallet.signPsbt(inscriber.toHex(), { isRevealTx: true })

      const inscribeTx = await this.datasource.relay({ hex: signedTxHex })
      log('inscribeTx:', inscribeTx)

      return {
        txid: inscribeTx,
        index: 0,
      }
    } else {
      throw new Error('inscriber is not ready')
    }
  }

  protected async depositRevealFee(
    revealed: {
      address: string
      revealFee: number
    },
    opts?: InscribeOptions,
  ): Promise<UTXOLimited[]> {
    if (!this.fundingWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    let spendables = await this.datasource.getSpendables({
      address: this.fundingWallet.selectedAddress,
      value: revealed.revealFee,
    })
    let totalSpendableValue = spendables.reduce((sum, utxo) => sum + utxo.sats, 0)

    while (totalSpendableValue < revealed.revealFee) {
      console.warn(`Insufficient funds in funding wallet. Waiting for more funds...`)
      await sleep(5000) // Wait for 5 seconds before retrying
      spendables = await this.datasource.getSpendables({
        address: this.fundingWallet.selectedAddress,
        value: revealed.revealFee,
      })
      totalSpendableValue = spendables.reduce((sum, utxo) => sum + utxo.sats, 0)
    }

    const psbt = await ordit.transactions.createPsbt({
      pubKey: this.fundingWallet.publicKey,
      address: this.fundingWallet.selectedAddress,
      outputs: [
        {
          address: revealed.address,
          value: revealed.revealFee,
        },
      ],
      network: this.fundingWallet.network,
      datasource: this.datasource,
      satsPerByte: opts?.commit_fee_rate || opts?.fee_rate || 1,
    })

    const signedTxHex = await this.fundingWallet.signPsbt(psbt.hex)

    log('depositRevealFee prepare relay:', signedTxHex)
    const txId = await this.datasource.relay({ hex: signedTxHex })

    log('depositRevealFee txId:', txId)
    return decodeUTXOs(signedTxHex, this.network, revealed.address)
  }

  public async generator(
    name: string,
    wasmBytes: Uint8Array,
    opts?: InscribeOptions,
  ): Promise<InscriptionID> {
    const sft: SFTRecord = {
      op: 'mint',
      tick: 'generator',
      amount: 1,
      attributes: {
        name: name,
      },
      content: {
        content_type: 'application/wasm',
        body: wasmBytes,
      },
    }

    return await this.inscribe(sft, opts)
  }

  public async deploy(
    tick: string,
    max: number,
    generator: Generator,
    opts?: DeployOptions | undefined,
  ): Promise<InscriptionID> {
    const sft: SFTRecord = {
      op: 'deploy',
      tick: tick,
      amount: max,
      attributes: {
        repeat: opts?.repeat || 0,
        generator: `/inscription/${inscriptionIDToString(generator)}`,
        deploy_args: opts?.deploy_args,
      },
    }

    return await this.inscribe(sft, opts)
  }

  public async mint(
    tickInscriptionId: InscriptionID,
    userInput: string,
    opts?: InscribeOptions,
  ): Promise<InscriptionID> {
    if (!opts?.satpoint) {
      throw new Error('mint must set satpoint')
    }

    let tick = await this.getTickByInscriptionId(tickInscriptionId)
    const generator = await this.generatorLoader.load(tick.generator)

    let seed_utxo = opts.satpoint.outpoint
    let seed_tx = await this.datasource.getTransaction({
      txId: seed_utxo.txid,
    })

    const seed = new InscribeSeed(seed_tx.tx.blockhash, seed_utxo)
    const sft = await generator.inscribeGenerate(tick.deploy_args, seed, userInput)
    log('SFT record:', sft)

    sft.op = 'mint'
    sft.tick = tick.tick

    return await this.inscribe(sft, opts)
  }

  private async getTickByInscriptionId(inscription_id: InscriptionID): Promise<Tick> {
    const tickInscription = await this.datasource.getInscription({
      id: inscriptionIDToString(inscription_id),
      decodeMetadata: true,
    })

    if (!tickInscription.meta) {
      throw new BitseedSDKError('tick meta is nil')
    }

    log('tickInscription.meta:', tickInscription.meta)

    const generatorInscriptionId = extractInscriptionID(
      tickInscription.meta.attributes['generator'],
    )
    if (!generatorInscriptionId) {
      throw new BitseedSDKError('generator inscriptionid is nil')
    }

    const tick: Tick = {
      tick: tickInscription.meta.tick,
      max: tickInscription.meta.amount,
      generator: generatorInscriptionId,
      repeat: tickInscription.meta.attributes['repeat'],
      deploy_args: tickInscription.meta.attributes['deploy_args'],
    }

    return tick
  }

  public async merge(_a: InscriptionID, _b: InscriptionID): Promise<InscriptionID> {
    throw new Error('Method not implemented.')
  }

  public async split(_a: InscriptionID): Promise<[InscriptionID, InscriptionID]> {
    throw new Error('Method not implemented.')
  }
}
