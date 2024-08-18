// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import cbor from 'cbor'
import * as bitcoin from 'bitcoinjs-lib'
import { Network } from '../../types/index.js'
import {
  IDatasource,
  GetBalanceOptions,
  GetInscriptionOptions,
  Inscription,
  GetInscriptionUTXOOptions,
  UTXO,
  GetInscriptionsOptions,
  GetTransactionOptions,
  Transaction,
  Vin,
  Vout,
  GetSpendablesOptions,
  UTXOLimited,
  GetUnspentsOptions,
  GetUnspentsResponse,
  RelayOptions,
} from '@sadoprotocol/ordit-sdk'
import {
  getRoochNodeUrl,
  RoochClient,
  IndexerStateIDView,
  PaginatedUTXOStateViews,
  RoochTransport,
  PaginatedInscriptionStateViews,
  InscriptionStateView,
  UTXOStateView,
  UTXOView,
  QueryInscriptionsParams,
  Args,
} from '@roochnetwork/rooch-sdk'
import { decodeScriptPubKey, hexStringToTxid } from '../../utils/index.js'

type RoochDataSourceOptions = {
  network?: Network
  url?: string
  transport?: RoochTransport
}

export class RoochDataSource implements IDatasource {
  private network: bitcoin.Network
  private roochClient: RoochClient

  constructor(opts: RoochDataSourceOptions) {
    if (opts.transport != null) {
      this.network = bitcoin.networks.regtest
      this.roochClient = new RoochClient({
        transport: opts.transport,
      })

      return
    }

    if (opts.url != null) {
      this.network = bitcoin.networks.regtest
      this.roochClient = new RoochClient({
        url: opts.url,
      })

      return
    }

    if (opts.network != null) {
      this.network = toBitcoinNetwork(opts.network)
      let roochNetwork = bitcoinNetworkToRooch(opts.network)
      let nodeURL = getRoochNodeUrl(roochNetwork)
      this.roochClient = new RoochClient({
        url: nodeURL,
      })

      return
    }

    throw new Error('not support RoochDataSource Options')
  }

  async getBalance({ address }: GetBalanceOptions): Promise<number> {
    let totalBalance = 0n
    let cursor: IndexerStateIDView | null = null
    const limit = 100

    while (true) {
      const response: PaginatedUTXOStateViews = await this.roochClient.queryUTXO({
        filter: {
          owner: address,
        },
        cursor: cursor,
        limit: limit.toString(),
      })

      for (const utxo of response.data) {
        totalBalance += BigInt(utxo.value.value)
      }

      if (!response.has_next_page || !response.next_cursor) {
        break
      }

      cursor = response.next_cursor
    }

    return Number(totalBalance)
  }

  async getInscription({ id, decodeMetadata }: GetInscriptionOptions): Promise<Inscription> {
    const response: PaginatedInscriptionStateViews = await this.roochClient.queryInscriptions({
      filter: {
        inscription_id: {
          txid: id.split('i')[0],
          index: parseInt(id.split('i')[1]),
        },
      },
      limit: '1',
    })

    if (response.data.length === 0) {
      throw new Error(`Inscription with id ${id} not found`)
    }

    const inscriptionState: InscriptionStateView = response.data[0]
    const inscriptionView = inscriptionState.value

    let body: Buffer | null = null
    if (inscriptionView.body) {
      const bodyHex = inscriptionView.body.startsWith('0x')
        ? inscriptionView.body.slice(2)
        : inscriptionView.body
      body = Buffer.from(bodyHex, 'hex')
    }

    // Convert the Rooch inscription state to the Inscription type expected by IDatasource
    const inscription: Inscription = {
      id: `${inscriptionView.bitcoin_txid}i${inscriptionView.index}`,
      number: inscriptionView.inscription_number,
      owner: inscriptionState.owner ?? '',
      mediaContent: body ? Buffer.from(body).toString('base64') : '',
      mediaSize: body ? body.length : 0,
      mediaType: inscriptionView.content_type ?? '',
      timestamp: new Date(inscriptionState.created_at).getTime(),
      genesis: inscriptionView.bitcoin_txid,
      outpoint: `${inscriptionView.txid}:${inscriptionView.offset}`,
      fee: 0,
      height: 0,
      sat: 0,
    }

    if (decodeMetadata && inscriptionView.metadata) {
      try {
        // Decode the hex-encoded metadata
        const metadataHex = inscriptionView.metadata.startsWith('0x')
          ? inscriptionView.metadata.slice(2)
          : inscriptionView.metadata
        const metadataBuffer = Buffer.from(metadataHex, 'hex')
        // Decode the CBOR data
        const decodedMetadata = cbor.decode(metadataBuffer)
        inscription.meta = decodedMetadata
      } catch (error) {
        console.warn(`Failed to decode CBOR metadata for inscription ${id}: ${error}`)
      }
    }

    return inscription
  }

  async getInscriptionUTXO({ id }: GetInscriptionUTXOOptions): Promise<UTXO> {
    // Get inscription information first
    const inscription = await this.getInscription({ id, decodeMetadata: false })

    // Parse the outpoint from the inscription
    const [txid, voutStr] = inscription.outpoint.split(':')
    const vout = parseInt(voutStr, 10)

    // Query UTXO using Rooch SDK with out_point filter
    const response = await this.roochClient.queryUTXO({
      filter: {
        out_point: {
          txid,
          vout,
        },
      },
      limit: '1',
    })

    if (response.data.length === 0) {
      throw new Error(`UTXO for inscription ${id} not found`)
    }

    const utxoState: UTXOStateView = response.data[0]
    const utxoValue: UTXOView = utxoState.value

    let sats: number
    try {
      const bigIntValue = BigInt(utxoValue.value)
      if (bigIntValue > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new Error('UTXO value exceeds safe integer range')
      }
      sats = Number(bigIntValue)
    } catch (error) {
      throw new Error(`Failed to convert UTXO value to number: ${error}`)
    }

    // Convert Rooch UTXO data to required UTXO type
    const utxo: UTXO = {
      n: utxoValue.vout,
      txid: utxoValue.bitcoin_txid,
      sats,
      scriptPubKey: {
        asm: '', // Rooch does not provide this information
        desc: '',
        hex: '',
        address: utxoState.owner_bitcoin_address || utxoState.owner,
        type: 'p2tr', // Assuming all inscriptions use Taproot
      },
      safeToSpend: true, // Assuming all queried UTXOs are safe to spend
      confirmation: -1, // Rooch does not provide this information
    }

    return utxo
  }

  async getInscriptions({
    creator,
    owner,
    mimeType,
    mimeSubType,
    outpoint,
    sort,
    limit,
    next,
    decodeMetadata,
  }: GetInscriptionsOptions): Promise<Inscription[]> {
    const inscriptions: Inscription[] = []
    let cursor: IndexerStateIDView | null = next
      ? { state_index: next.split(':')[0], tx_order: next.split(':')[1] }
      : null
    const pageLimit = Math.min(limit || 100, 100) // Max 100 per page

    // Check for unsupported filter types
    if (creator || mimeType || mimeSubType || outpoint) {
      throw new Error(
        'Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch',
      )
    }

    while (inscriptions.length < (limit || Infinity)) {
      const queryParams: QueryInscriptionsParams = {
        filter: 'all',
        cursor,
        limit: pageLimit.toString(),
        descendingOrder: sort === 'desc',
      }

      if (owner) {
        queryParams.filter = { owner }
      }

      const response = await this.roochClient.queryInscriptions(queryParams)

      for (const inscriptionState of response.data) {
        const inscription = this.convertToInscription(inscriptionState, decodeMetadata)
        inscriptions.push(inscription)

        if (inscriptions.length >= (limit || Infinity)) break
      }

      if (!response.has_next_page || !response.next_cursor) break
      cursor = response.next_cursor
    }

    return inscriptions
  }

  private convertToInscription(
    inscriptionState: InscriptionStateView,
    decodeMetadata: boolean | undefined,
  ): Inscription {
    const inscriptionView = inscriptionState.value

    let body: Buffer | null = null
    if (inscriptionView.body) {
      const bodyHex = inscriptionView.body.startsWith('0x')
        ? inscriptionView.body.slice(2)
        : inscriptionView.body
      body = Buffer.from(bodyHex, 'hex')
    }

    const inscription: Inscription = {
      id: `${inscriptionView.bitcoin_txid}i${inscriptionView.index}`,
      outpoint: `${inscriptionView.txid}:${inscriptionView.offset}`,
      owner: inscriptionState.owner ?? '',
      genesis: inscriptionView.bitcoin_txid,
      fee: 0, // Rooch doesn't provide this information
      height: 0, // Rooch doesn't provide this information
      number: inscriptionView.inscription_number,
      sat: 0, // Rooch doesn't provide this information
      timestamp: new Date(inscriptionState.created_at).getTime(),
      mediaType: inscriptionView.content_type ?? '',
      mediaContent: body ? Buffer.from(body).toString('base64') : '',
      mediaSize: body ? body.length : 0,
    }

    if (decodeMetadata && inscriptionView.metadata) {
      try {
        const metadataHex = inscriptionView.metadata.startsWith('0x')
          ? inscriptionView.metadata.slice(2)
          : inscriptionView.metadata
        const metadataBuffer = Buffer.from(metadataHex, 'hex')
        inscription.meta = cbor.decode(metadataBuffer)
      } catch (error) {
        console.warn(`Failed to decode CBOR metadata for inscription ${inscription.id}: ${error}`)
      }
    }

    return inscription
  }

  async getTransaction({ txId, hex }: GetTransactionOptions): Promise<{
    tx: Transaction
  }> {
    // Get transaction information
    const txResult = (await this.roochClient.executeViewFunction({
      target: '0x4::bitcoin::get_tx',
      typeArgs: [],
      args: [Args.address(txId)],
    })) as any

    if (!txResult.return_values || txResult.return_values.length === 0) {
      throw new Error(`Transaction with id ${txId} not found`)
    }

    const btcTxOption = txResult.return_values[0].decoded_value.value.vec[0]
    if (!btcTxOption) {
      throw new Error(`Transaction with id ${txId} not found`)
    }

    const btcTx = btcTxOption.value

    // Convert Rooch BTC transaction to the required Transaction type
    const tx: Transaction = {
      txid: btcTx.id,
      hash: btcTx.id, // Assuming hash is the same as txid
      version: btcTx.version,
      size: 0, // Not available in Rooch BTC tx
      vsize: 0, // Not available in Rooch BTC tx
      weight: 0, // Not available in Rooch BTC tx
      locktime: btcTx.lock_time,
      vin: btcTx.input.map(
        (input: any): Vin => ({
          txid: input.value.previous_output.value.txid,
          vout: input.value.previous_output.value.vout,
          scriptSig: {
            asm: '', // Not available in Rooch BTC tx
            hex: input.value.script_sig.slice(2), // Remove '0x' prefix
          },
          txinwitness: input.value.witness.value.witness.map((w: string) => w.slice(2)), // Remove '0x' prefix
          sequence: input.value.sequence,
          value: 0, // Not available in Rooch BTC tx
        }),
      ),
      vout: btcTx.output.map((output: any, index: number): Vout => {
        const scriptPubHex = output.value.script_pubkey.value.bytes.slice(2) // Remove '0x' prefix
        const scriptPubKey = decodeScriptPubKey(scriptPubHex, this.network)

        return {
          value: Number(output.value.value),
          n: index,
          ordinals: [], // Not available in Rooch BTC tx
          inscriptions: [], // Not available in Rooch BTC tx
          spent: false, // Not available in Rooch BTC tx
          sats: Number(output.value.value),
          scriptPubKey: scriptPubKey,
        }
      }),
      blockhash: '', // Will be filled later
      blockheight: 0, // Will be filled later
      blocktime: 0, // Will be filled later
      confirmations: 0, // Not available in Rooch BTC tx
      time: 0, // Not available in Rooch BTC tx
      fee: 0, // Not available in Rooch BTC tx
    }

    if (hex) {
      tx.hex = '' // Not available in Rooch BTC tx
    }

    // Get transaction height
    const txHeightResult = (await this.roochClient.executeViewFunction({
      target: '0x4::bitcoin::get_tx_height',
      typeArgs: [],
      args: [Args.address(txId)],
    })) as any

    if (
      txHeightResult.result &&
      txHeightResult.result.return_values &&
      txHeightResult.result.return_values.length > 0
    ) {
      const blockHeight = Number(txHeightResult.result.return_values[0].decoded_value)
      tx.blockheight = blockHeight

      // Get block information
      const blockResult = (await this.roochClient.executeViewFunction({
        target: '0x4::bitcoin::get_block_by_height',
        typeArgs: [],
        args: [Args.u64(BigInt(blockHeight))],
      })) as any

      if (
        blockResult.result &&
        blockResult.result.return_values &&
        blockResult.result.return_values.length > 0
      ) {
        const block = blockResult.result.return_values[0].decoded_value
        tx.blockhash = block.prev_blockhash
        tx.blocktime = Number(block.time)
      }
    }

    return { tx }
  }

  async getSpendables({
    address,
    value,
    type = 'all',
    rarity,
    filter,
    limit = 100,
  }: GetSpendablesOptions): Promise<UTXOLimited[]> {
    if (rarity !== undefined) {
      throw new Error('Rarity options are not supported for Rooch getSpendables')
    }

    if (filter !== undefined && filter.length > 0) {
      throw new Error('filter options are not supported for Rooch getSpendables')
    }

    if (!address || typeof address !== 'string') {
      throw new Error('Invalid address provided')
    }

    if (typeof value !== 'number' || value < 0) {
      throw new Error('Invalid value provided')
    }

    const spendables: ExtendedUTXOLimited[] = []
    let cursor: IndexerStateIDView | null = null
    let totalSats = 0

    while (totalSats < value && spendables.length < limit) {
      const response: PaginatedUTXOStateViews = await this.roochClient.queryUTXO({
        filter: {
          owner: address,
        },
        cursor: cursor,
        limit: Math.min(limit - spendables.length, 100).toString(),
      })

      for (const utxoState of response.data) {
        const utxo = await this.convertToUTXOLimited(utxoState)

        if (this.isSpendable(utxo, type)) {
          spendables.push(utxo)
          totalSats += utxo.sats

          if (totalSats >= value || spendables.length >= limit) {
            break
          }
        }
      }

      if (!response.has_next_page || !response.next_cursor) {
        break
      }

      cursor = response.next_cursor
    }

    return spendables
  }

  private async convertToUTXOLimited(utxoState: UTXOStateView): Promise<ExtendedUTXOLimited> {
    const utxoValue: UTXOView = utxoState.value

    if (!utxoValue.bitcoin_txid || !utxoValue.value || typeof utxoValue.vout !== 'number') {
      throw new Error('Invalid UTXO data')
    }

    const output = await this.getTransaction({
      txId: hexStringToTxid(utxoValue.bitcoin_txid),
      hex: true,
    })

    const utxoOuts = output.tx.vout.filter((out) => {
      return out.n == utxoValue.vout
    })

    if (utxoOuts.length == 0) {
      throw new Error('Invalid UTXO scriptPubKey')
    }

    const scriptPubKey = utxoOuts[0].scriptPubKey

    return {
      txid: utxoValue.bitcoin_txid,
      n: utxoValue.vout,
      sats: this.safeParseBigInt(utxoValue.value),
      scriptPubKey: {
        asm: scriptPubKey.asm,
        desc: scriptPubKey.desc,
        hex: scriptPubKey.hex,
        address: scriptPubKey.address || '',
        type: scriptPubKey.type,
      },
      seals: utxoValue.seals,
    }
  }

  private safeParseBigInt(value: string): number {
    try {
      const bigIntValue = BigInt(value)
      if (bigIntValue > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new Error('UTXO value exceeds safe integer range')
      }
      return Number(bigIntValue)
    } catch (error) {
      console.error(`Failed to parse UTXO value: ${value}`)
      throw new Error(`Invalid UTXO value: ${value}`)
    }
  }

  private isSpendable(utxo: ExtendedUTXOLimited, type: 'all' | 'spendable'): boolean {
    if (type === 'spendable') {
      return this.isUTXOSpendable(utxo)
    }
    return true
  }

  private isUTXOSpendable(utxo: ExtendedUTXOLimited): boolean {
    return utxo.seals === null || utxo.seals === undefined || Object.keys(utxo.seals).length === 0
  }

  async getUnspents({
    address,
    type = 'all',
    rarity,
    sort = 'desc',
    limit = 100,
    next,
  }: GetUnspentsOptions): Promise<GetUnspentsResponse> {
    if (rarity !== undefined) {
      throw new Error('Rarity options are not supported for Rooch getUnspents')
    }

    if (!address || typeof address !== 'string') {
      throw new Error('Invalid address provided')
    }

    let spendableUTXOs: UTXO[] = []
    let unspendableUTXOs: UTXO[] = []
    let totalUTXOs = 0
    let cursor: IndexerStateIDView | null = next ? JSON.parse(next) : null

    while (spendableUTXOs.length + unspendableUTXOs.length < limit) {
      const response: PaginatedUTXOStateViews = await this.roochClient.queryUTXO({
        filter: { owner: address },
        cursor,
        limit: Math.min(limit - (spendableUTXOs.length + unspendableUTXOs.length), 100).toString(),
      })

      for (const utxoState of response.data) {
        const utxo = await this.convertToUTXO(utxoState)

        if (utxo.safeToSpend) {
          spendableUTXOs.push(utxo)
        } else {
          unspendableUTXOs.push(utxo)
        }

        totalUTXOs++

        if (spendableUTXOs.length + unspendableUTXOs.length >= limit) {
          break
        }
      }

      if (!response.has_next_page || !response.next_cursor) {
        break
      }

      cursor = response.next_cursor
    }

    // Apply sorting
    const sortFunction = (a: UTXO, b: UTXO) => {
      return sort === 'asc' ? a.sats - b.sats : b.sats - a.sats
    }

    spendableUTXOs.sort(sortFunction)
    unspendableUTXOs.sort(sortFunction)

    // Apply type filter
    if (type === 'spendable') {
      unspendableUTXOs = []
    }

    return {
      totalUTXOs,
      spendableUTXOs,
      unspendableUTXOs,
    }
  }

  private async convertToUTXO(utxoState: UTXOStateView): Promise<UTXO> {
    const limitedUTXO = await this.convertToUTXOLimited(utxoState)

    return {
      ...limitedUTXO,
      safeToSpend: this.isUTXOSpendable(limitedUTXO),
      confirmation: -1, // Not available in Rooch
    }
  }

  async relay({ hex, maxFeeRate, validate }: RelayOptions): Promise<string> {
    if (validate !== undefined) {
      throw new Error('validate options are not supported for Rooch broadcastBitcoinTX')
    }

    if (!hex || typeof hex !== 'string' || !isValidHex(hex)) {
      throw new Error('Invalid transaction hex')
    }

    try {
      const response = await this.roochClient.broadcastBitcoinTX({
        hex,
        maxfeerate: maxFeeRate ?? undefined,
        maxburnamount: undefined,
      })

      return response
    } catch (error) {
      throw new Error(`Failed to broadcast transaction: ${error}`)
    }
  }
}

function toBitcoinNetwork(network: Network): bitcoin.Network {
  switch (network) {
    case 'mainnet':
      return bitcoin.networks.bitcoin
    case 'testnet':
      return bitcoin.networks.testnet
    case 'regtest':
      return bitcoin.networks.regtest
    default:
      throw new Error(`Unknown network: ${network}`)
  }
}

function bitcoinNetworkToRooch(network: Network): 'testnet' | 'devnet' | 'localnet' {
  switch (network) {
    case 'testnet':
      return 'testnet'
    case 'regtest':
      return 'localnet'
    default:
      throw new Error(`Unknown network: ${network}`)
  }
}

function isValidHex(hex: string): boolean {
  const hexRegex = /^[0-9A-Fa-f]+$/
  return hexRegex.test(hex)
}

interface ExtendedUTXOLimited extends UTXOLimited {
  seals: { [key: string]: string[] }
}
