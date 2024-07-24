import cbor from 'cbor'
import { Network } from '../../types'
import {
  GetBalanceOptions,
  GetInscriptionOptions,
  Inscription,
  GetInscriptionUTXOOptions,
  UTXO,
  GetInscriptionsOptions,
  GetTransactionOptions,
  Transaction
} from "@sadoprotocol/ordit-sdk";
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
  Args
} from '@roochnetwork/rooch-sdk';
import {
  RoochBTCTransaction
} from './bitcoin_types';

interface RoochDataSourceOptions {
  network: Network;
  transport?: RoochTransport
}

export class RoochDataSource /*implements IDatasource*/ {
  private roochClient: RoochClient;

  constructor(opts: RoochDataSourceOptions) {
    if (opts.transport != null) {
      this.roochClient = new RoochClient({
        transport: opts.transport
      });

      return
    }

    let roochNetwork = bitcoinNetworkToRooch(opts.network)
    let nodeURL = getRoochNodeUrl(roochNetwork);
    this.roochClient = new RoochClient({
      url: nodeURL,
    });
  }

  async getBalance({ address }: GetBalanceOptions): Promise<number> {
    let totalBalance = 0n;
    let cursor: IndexerStateIDView | null = null;
    const limit = 100;

    while (true) {
      const response: PaginatedUTXOStateViews = await this.roochClient.queryUTXO({
        filter: {
          owner: address,
        },
        cursor: cursor,
        limit: limit.toString(),
      });

      for (const utxo of response.data) {
        totalBalance += BigInt(utxo.value.value);
      }

      if (!response.has_next_page || !response.next_cursor) {
        break;
      }

      cursor = response.next_cursor;
    }

    return Number(totalBalance);
  }

  async getInscription({ id, decodeMetadata }: GetInscriptionOptions): Promise<Inscription> {
    const response: PaginatedInscriptionStateViews = await this.roochClient.queryInscriptions({
      filter: {
        inscription_id: {
          txid: id.split('i')[0],
          index: parseInt(id.split('i')[1])
        }
      },
      limit: "1"
    });

    if (response.data.length === 0) {
      throw new Error(`Inscription with id ${id} not found`);
    }

    const inscriptionState: InscriptionStateView = response.data[0];
    const inscriptionView = inscriptionState.value;

    // Convert the Rooch inscription state to the Inscription type expected by IDatasource
    const inscription: Inscription = {
      id: `${inscriptionView.bitcoin_txid}i${inscriptionView.index}`,
      number: inscriptionView.inscription_number,
      owner: inscriptionState.owner ?? "",
      mediaContent: inscriptionView.body ?? "",
      mediaSize: inscriptionView.body ? Buffer.from(inscriptionView.body, 'base64').length : 0,
      mediaType: inscriptionView.content_type ?? "",
      timestamp: new Date(inscriptionState.created_at).getTime(),
      genesis: inscriptionView.bitcoin_txid,
      outpoint: `${inscriptionView.txid}:${inscriptionView.offset}`,
      fee: 0,
      height: 0,
      sat: 0
    };

    if (decodeMetadata && inscriptionView.metadata) {
      try {
        // Decode the base64-encoded metadata
        const metadataBuffer = Buffer.from(inscriptionView.metadata, 'base64');
        // Decode the CBOR data
        const decodedMetadata = cbor.decode(metadataBuffer);
        inscription.meta = decodedMetadata;
      } catch (error) {
        console.warn(`Failed to decode CBOR metadata for inscription ${id}: ${error}`);
      }
    }

    return inscription;
  }

  async getInscriptionUTXO({ id }: GetInscriptionUTXOOptions): Promise<UTXO> {
    // Get inscription information first
    const inscription = await this.getInscription({ id, decodeMetadata: false });
  
    // Parse the outpoint from the inscription
    const [txid, voutStr] = inscription.outpoint.split(':');
    const vout = parseInt(voutStr, 10);
  
    // Query UTXO using Rooch SDK with out_point filter
    const response = await this.roochClient.queryUTXO({
      filter: {
        out_point: {
          txid,
          vout
        }
      },
      limit: "1"
    });
  
    if (response.data.length === 0) {
      throw new Error(`UTXO for inscription ${id} not found`);
    }
  
    const utxoState: UTXOStateView = response.data[0];
    const utxoValue: UTXOView = utxoState.value;
  
    let sats: number;
    try {
      const bigIntValue = BigInt(utxoValue.value);
      if (bigIntValue > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new Error("UTXO value exceeds safe integer range");
      }
      sats = Number(bigIntValue);
    } catch (error) {
      throw new Error(`Failed to convert UTXO value to number: ${error}`);
    }

    // Convert Rooch UTXO data to required UTXO type
    const utxo: UTXO = {
      n: utxoValue.vout,
      txid: utxoValue.bitcoin_txid,
      sats,
      scriptPubKey: {
        asm: "", // Rooch does not provide this information
        desc: "",
        hex: "",
        address: utxoState.owner_bitcoin_address || utxoState.owner,
        type: "p2tr", // Assuming all inscriptions use Taproot
      },
      safeToSpend: true, // Assuming all queried UTXOs are safe to spend
      confirmation: -1, // Rooch does not provide this information
    };
  
    return utxo;
  }

  async getInscriptions({ creator, owner, mimeType, mimeSubType, outpoint, sort, limit, next, decodeMetadata }: GetInscriptionsOptions): Promise<Inscription[]> {
    const inscriptions: Inscription[] = [];
    let cursor: IndexerStateIDView | null = next 
      ? { state_index: next.split(':')[0], tx_order: next.split(':')[1] } 
      : null;
    const pageLimit = Math.min(limit || 100, 100); // Max 100 per page
  
    // Check for unsupported filter types
    if (creator || mimeType || mimeSubType || outpoint) {
      throw new Error("Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch");
    }
  
    while (inscriptions.length < (limit || Infinity)) {
      const queryParams: QueryInscriptionsParams = {
        filter: 'all',
        cursor,
        limit: pageLimit.toString(),
        descendingOrder: sort === 'desc'
      };
  
      if (owner) {
        queryParams.filter = { owner };
      }
  
      const response = await this.roochClient.queryInscriptions(queryParams);
  
      for (const inscriptionState of response.data) {
        const inscription = this.convertToInscription(inscriptionState, decodeMetadata);
        inscriptions.push(inscription);
  
        if (inscriptions.length >= (limit || Infinity)) break;
      }
  
      if (!response.has_next_page || !response.next_cursor) break;
      cursor = response.next_cursor;
    }
  
    return inscriptions;
  }
  
  private convertToInscription(inscriptionState: InscriptionStateView, decodeMetadata: boolean | undefined): Inscription {
    const inscriptionView = inscriptionState.value;
    const inscription: Inscription = {
      id: `${inscriptionView.bitcoin_txid}i${inscriptionView.index}`,
      outpoint: `${inscriptionView.txid}:${inscriptionView.offset}`,
      owner: inscriptionState.owner ?? "",
      genesis: inscriptionView.bitcoin_txid,
      fee: 0, // Rooch doesn't provide this information
      height: 0, // Rooch doesn't provide this information
      number: inscriptionView.inscription_number,
      sat: 0, // Rooch doesn't provide this information
      timestamp: new Date(inscriptionState.created_at).getTime(),
      mediaType: inscriptionView.content_type ?? "",
      mediaSize: inscriptionView.body ? Buffer.from(inscriptionView.body, 'base64').length : 0,
      mediaContent: inscriptionView.body ?? "",
    };
  
    if (decodeMetadata && inscriptionView.metadata) {
      try {
        const metadataBuffer = Buffer.from(inscriptionView.metadata, 'base64');
        inscription.meta = cbor.decode(metadataBuffer);
      } catch (error) {
        console.warn(`Failed to decode CBOR metadata for inscription ${inscription.id}: ${error}`);
      }
    }
  
    return inscription;
  }

  async getTransaction({ txId, ordinals, hex, witness, decodeMetadata }: GetTransactionOptions): Promise<{
    tx: Transaction;
  }> {
    // Get transaction information
    const txResult = await this.roochClient.executeViewFunction({
      target: '0x1::bitcoin::get_tx',
      typeArgs: [],
      args: [Args.address(txId)],
    });

    if (!txResult.return_values || txResult.return_values.length === 0) {
      throw new Error(`Transaction with id ${txId} not found`);
    }

    const btcTx = txResult.return_values[0].decoded_value as any;

    // Convert Rooch BTC transaction to the required Transaction type
    const tx: Transaction = {
      txid: btcTx.id,
      version: Number(btcTx.version),
      locktime: Number(btcTx.lock_time),
      vin: btcTx.input.map((input: any) => ({
        txid: input.previous_output.txid,
        vout: Number(input.previous_output.vout),
        scriptSig: {
          asm: "", // Not available in Rooch BTC tx
          hex: Buffer.from(input.script_sig).toString('hex')
        },
        sequence: Number(input.sequence),
        witness: input.witness.witness.map((w: any) => Buffer.from(w).toString('hex'))
      })),
      vout: btcTx.output.map((output: any, index: number) => ({
        value: Number(output.value),
        n: index,
        scriptPubKey: {
          asm: "", // Not available in Rooch BTC tx
          hex: Buffer.from(output.script_pubkey).toString('hex'),
          address: output.recipient_address // Assuming this is available
        }
      })),
      size: 0, // Not available in Rooch BTC tx
      weight: 0, // Not available in Rooch BTC tx
      fee: BigInt(0), // Not available in Rooch BTC tx
      status: {
        confirmed: true, // Assuming all returned transactions are confirmed
        block_height: undefined,
        block_hash: undefined,
        block_time: undefined
      }
    };

    if (decodeMetadata && btcTx.metadata) {
      try {
        const metadataBuffer = Buffer.from(btcTx.metadata, 'base64');
        tx.meta = cbor.decode(metadataBuffer);
      } catch (error) {
        console.warn(`Failed to decode CBOR metadata for transaction ${txId}: ${error}`);
      }
    }

    // Get transaction height
    const txHeightResult = await this.roochClient.executeViewFunction({
      target: '0x1::bitcoin::get_tx_height',
      typeArgs: [],
      args: [Args.address(txId)],
    });

    if (txHeightResult.return_values && txHeightResult.return_values.length > 0) {
      const blockHeight = Number(txHeightResult.return_values[0].decoded_value);
      tx.status.block_height = blockHeight;

      // Get block information
      const blockResult = await this.roochClient.executeViewFunction({
        target: '0x1::bitcoin::get_block_by_height',
        typeArgs: [],
        args: [Args.u64(BigInt(blockHeight))],
      });

      if (blockResult.return_values && blockResult.return_values.length > 0) {
        const block = blockResult.return_values[0].decoded_value;
        tx.status.block_hash = block.prev_blockhash;
        tx.status.block_time = Number(block.time);
      }
    }

    // Note: We can't provide hex or witness information as it's not available in the Rooch BTC tx representation

    return { tx };
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