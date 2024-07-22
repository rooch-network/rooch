import cbor from 'cbor'
import { Network } from '../../types'
import {
  GetBalanceOptions,
  GetInscriptionOptions,
  Inscription
} from "@sadoprotocol/ordit-sdk";
import { 
  getRoochNodeUrl, 
  RoochClient, 
  IndexerStateIDView, 
  PaginatedUTXOStateViews, 
  RoochTransport,
  PaginatedInscriptionStateViews,
  InscriptionStateView
} from '@roochnetwork/rooch-sdk';

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