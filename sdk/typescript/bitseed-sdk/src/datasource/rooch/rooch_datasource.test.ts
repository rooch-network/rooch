// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { describe, it, vi, expect, beforeEach } from 'vitest'
import cbor from 'cbor'
import { Buffer } from 'buffer'
import { RoochDataSource } from './rooch_datasource.js';
import { Wallet } from '../../wallet/index.js';
import { 
  RoochTransport, 
  PaginatedUTXOStateViews, 
  PaginatedInscriptionStateViews,
  UTXOStateView,
} from '@roochnetwork/rooch-sdk';

class MockRoochTransport implements RoochTransport {
  private mockResponses: Map<string, any[]> = new Map();
  private mockErrors: Map<string, Error[]> = new Map();
  private callCounts: Map<string, number> = new Map();

  setMockResponse(method: string, response: any) {
    if (!this.mockResponses.has(method)) {
      this.mockResponses.set(method, []);
    }
    this.mockResponses.get(method)!.push(response);
    this.callCounts.set(method, 0);
  }

  setMockErrorResponse(method: string, error: Error) {
    if (!this.mockErrors.has(method)) {
      this.mockErrors.set(method, []);
    }
    this.mockErrors.get(method)!.push(error);
    this.callCounts.set(method, 0);
  }

  async request<T>({ method }: { method: string; params: unknown[] }): Promise<T> {
    const callCount = this.callCounts.get(method) || 0;
    this.callCounts.set(method, callCount + 1);

    const errors = this.mockErrors.get(method);
    if (errors && errors.length > 0) {
      const error = errors[callCount % errors.length];
      throw error;
    }

    const responses = this.mockResponses.get(method);
    if (responses && responses.length > 0) {
      const response = responses[callCount % responses.length];
      return response as T;
    }

    throw new Error(`No mock response or error set for method: ${method}`);
  }

  resetMocks() {
    this.mockResponses.clear();
    this.mockErrors.clear();
    this.callCounts.clear();
  }
}

describe('RoochDataSource', () => {
  let instance: RoochDataSource;
  let wallet: Wallet;
  let mockTransport: MockRoochTransport;

  beforeEach(() => {
    wallet = new Wallet({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: "testnet",
      type: 'taproot',
    });

    mockTransport = new MockRoochTransport();
    instance = new RoochDataSource({ network: 'testnet', transport: mockTransport });
  });

  describe('getBalance', () => {
    it('should return the correct balance for a given address', async () => {
      if (!wallet.selectedAddress) {
        throw new Error('no selected address');
      }

      const mockResponse: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              value: '1000000',
              bitcoin_txid: 'mock_txid',
              seals: {'mock_seals':[]},
              txid: 'mock_txid',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            owner: wallet.selectedAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockResponse);

      const balance = await instance.getBalance({ address: wallet.selectedAddress });
      expect(balance).toBe(1000000);
    });

    it('should return 0 balance for a new address', async () => {
      const wallet = new Wallet({
        bip39: 'right second until palace kid wear tennis phone bike broccoli oval saddle',
        network: "testnet",
        type: 'taproot',
      });

      const account = wallet.generateAddress('taproot', 2, 0);
      if (!account.address) {
        throw new Error('no selected address');
      }

      const mockResponse: PaginatedUTXOStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockResponse);

      const balance = await instance.getBalance({ address: account.address });
      expect(balance).toBe(0);
    });
  });
  
  describe('getInscription', () => {
    it('should successfully get an inscription without decoding metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: '0x6d6f636b426f6479', // hex encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: '0x6d6f636b4d65746164617461', // hex encoded 'mockMetadata'
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: false });

      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
    });

    it('should successfully get an inscription and decode metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockMetadata = {
        name: "Test Inscription",
        description: "This is a test inscription",
        attributes: [
          { trait_type: "Type", value: "Test" },
          { trait_type: "Version", value: "1.0" }
        ]
      };
      
      // Encode the metadata using CBOR
      const encodedMetadata = cbor.encode(mockMetadata);
      const base64EncodedMetadata = Buffer.from(encodedMetadata).toString('hex');

      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: '6d6f636b426f6479', // hex encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: base64EncodedMetadata,
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });

      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
        meta: mockMetadata,
      });
    });

    it('should successfully get an inscription and decode metadata with bitseed', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const base64EncodedMetadata = "0xa4626f70666465706c6f79647469636b646d6f766566616d6f756e741903e86a61747472696275746573a366726570656174016967656e657261746f7278202f696e736372697074696f6e2f756e646566696e656469756e646566696e65646b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";

      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: '6d6f636b426f6479', // hex encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: base64EncodedMetadata,
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });

      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
        meta: {
          amount: 1000,
          attributes:  {
            deploy_args: [
              "{\"height\":{\"type\":\"range\",\"data\":{\"min\":1,\"max\":1000}}}",
            ],
          generator: "/inscription/undefinediundefined",
            repeat: 1,
          },
          op: "deploy",
          tick: "move",
        },
      });
    });

    it('should throw an error when trying to get a non-existent inscription', async () => {
      const mockInscriptionId = 'nonexistenttxidi0';
      
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      await expect(instance.getInscription({ id: mockInscriptionId, decodeMetadata: false }))
        .rejects.toThrow(`Inscription with id ${mockInscriptionId} not found`);
    });

    it('should handle metadata decoding failure', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const invalidEncodedMetadata = 'invalidCBORdata';

      vi.spyOn(console, 'warn').mockImplementation(() => {});
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: '0x6d6f636b426f6479', // hex encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: invalidEncodedMetadata,
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });
  
      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
  
      expect(console.warn).toHaveBeenCalledWith(expect.stringContaining('Failed to decode CBOR metadata for inscription mocktxidi0'));
    });
  
    it('should handle inscription without metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: '6d6f636b426f6479', // hex encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: "", 
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });
  
      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
    });
  });

  describe('getInscriptionUTXO', () => {
    it('should successfully get an inscription UTXO', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: {'mock_seals':[]},
          txid: 'mocktxid',
          value: '1000',
          vout: 0
        }
      };

      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });

      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });

      const result = await instance.getInscriptionUTXO({ id: mockInscriptionId });

      expect(result).toEqual({
        n: 0,
        txid: 'mocktxid',
        sats: 1000,
        scriptPubKey: {
          asm: '',
          desc: '',
          hex: '',
          address: 'mock_bitcoin_address',
          type: 'p2tr',
        },
        safeToSpend: true,
        confirmation: -1,
      });
    });

    it('should throw an error when inscription does not exist', async () => {
      const mockInscriptionId = 'nonexistenttxidi0';
    
      // Mock empty response for inscription query
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow(`Inscription with id ${mockInscriptionId} not found`);
    });

    it('should throw an error when UTXO does not exist', async () => {
      const mockInscriptionId = 'mocktxidi0';
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      // Mock empty response for UTXO query
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow(`UTXO for inscription ${mockInscriptionId} not found`);
    });

    it('should throw an error when UTXO value exceeds safe integer range', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: {'mock_seals':[]},
          txid: 'mocktxid',
          value: '9007199254740992', // Exceeds Number.MAX_SAFE_INTEGER
          vout: 0
        }
      };
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow('Failed to convert UTXO value to number: Error: UTXO value exceeds safe integer range');
    });

    it('should throw an error when UTXO value cannot be converted to a number', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: {'mock_seals':[]},
          txid: 'mocktxid',
          value: 'not-a-number', // Invalid value
          vout: 0
        }
      };
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow('Failed to convert UTXO value to number: SyntaxError: Cannot convert not-a-number to a BigInt');
    });
  });

  describe('getInscriptions', () => {
    it('should successfully get inscriptions filtered by owner', async () => {
      const mockOwner = 'mockOwner1';
      const mockInscriptions = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid1',
            index: 0,
            inscription_number: 1,
            body: '6d6f636b426f647931', // hex encoded 'mockBody1'
            content_type: 'text/plain',
            txid: 'mocktxid1',
            offset: '0',
            metadata: '',
            parents: '',
            is_curse: false,
            sequence_number: 0,
          },
          created_at: '2023-01-01T00:00:00Z',
          flag: 0,
          id: 'mock_id1',
          object_type: 'mock_type',
          size: '1',
          state_index: '0',
          tx_order: '0',
          updated_at: '2023-01-01T00:00:00Z'
        }
      ];
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: mockInscriptions,
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscriptions({
        owner: mockOwner,
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      });
  
      expect(result).toEqual([
        {
          id: 'mocktxid1i0',
          number: 1,
          owner: mockOwner,
          mediaContent: 'bW9ja0JvZHkx',
          mediaSize: 9,
          mediaType: 'text/plain',
          timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
          genesis: 'mocktxid1',
          outpoint: 'mocktxid1:0',
          fee: 0,
          height: 0,
          sat: 0,
        }
      ]);
    });

    it('should successfully get inscriptions with pagination', async () => {
      const mockOwner = 'mockOwner1';
      const mockInscriptionsPage1 = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid1',
            index: 0,
            inscription_number: 1,
            body: '6d6f636b426f647931', // hex encoded 'mockBody1'
            content_type: 'text/plain',
            txid: 'mocktxid1',
            offset: '0',
            metadata: '',
            parents: '',
            is_curse: false,
            sequence_number: 0,
          },
          created_at: '2023-01-01T00:00:00Z',
          flag: 0,
          id: 'mock_id1',
          object_type: 'mock_type',
          size: '1',
          state_index: '0',
          tx_order: '0',
          updated_at: '2023-01-01T00:00:00Z'
        }
      ];
  
      const mockInscriptionsPage2 = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid2',
            index: 1,
            inscription_number: 2,
            body: '6d6f636b426f647932', // hex encoded 'mockBody2'
            content_type: 'text/plain',
            txid: 'mocktxid2',
            offset: '0',
            metadata: '',
            parents: '',
            is_curse: false,
            sequence_number: 0,
          },
          created_at: '2023-01-02T00:00:00Z',
          flag: 0,
          id: 'mock_id2',
          object_type: 'mock_type',
          size: '1',
          state_index: '1',
          tx_order: '1',
          updated_at: '2023-01-02T00:00:00Z'
        }
      ];
  
      const mockResponsePage1: PaginatedInscriptionStateViews = {
        data: mockInscriptionsPage1,
        has_next_page: true,
        next_cursor: { state_index: '1', tx_order: '1' }
      };
  
      const mockResponsePage2: PaginatedInscriptionStateViews = {
        data: mockInscriptionsPage2,
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponsePage1);
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponsePage2);
  
      const result = await instance.getInscriptions({
        owner: mockOwner,
        limit: 2,
        decodeMetadata: false,
        sort: 'asc'
      });
  
      expect(result).toEqual([
        {
          id: 'mocktxid1i0',
          number: 1,
          owner: 'mockOwner1',
          mediaContent: 'bW9ja0JvZHkx',
          mediaSize: 9,
          mediaType: 'text/plain',
          timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
          genesis: 'mocktxid1',
          outpoint: 'mocktxid1:0',
          fee: 0,
          height: 0,
          sat: 0,
        },
        {
          id: 'mocktxid2i1',
          number: 2,
          owner: 'mockOwner1',
          mediaContent: 'bW9ja0JvZHky',
          mediaSize: 9,
          mediaType: 'text/plain',
          timestamp: new Date('2023-01-02T00:00:00Z').getTime(),
          genesis: 'mocktxid2',
          outpoint: 'mocktxid2:0',
          fee: 0,
          height: 0,
          sat: 0,
        }
      ]);
    });

    it('should successfully get inscriptions and decode metadata', async () => {
      const mockOwner = 'mockOwner1';
      const mockMetadata = {
        name: "Test Inscription",
        description: "This is a test inscription"
      };
      
      // Encode the metadata using CBOR
      const encodedMetadata = cbor.encode(mockMetadata);
      const base64EncodedMetadata = Buffer.from(encodedMetadata).toString('hex');
  
      const mockInscriptions = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid1',
            index: 0,
            inscription_number: 1,
            body: '6d6f636b426f647931', // hex encoded 'mockBody1'
            content_type: 'text/plain',
            txid: 'mocktxid1',
            offset: '0',
            metadata: base64EncodedMetadata,
            parents: '',
            is_curse: false,
            sequence_number: 0,
          },
          created_at: '2023-01-01T00:00:00Z',
          flag: 0,
          id: 'mock_id1',
          object_type: 'mock_type',
          size: '1',
          state_index: '0',
          tx_order: '0',
          updated_at: '2023-01-01T00:00:00Z'
        }
      ];
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: mockInscriptions,
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscriptions({
        limit: 10,
        decodeMetadata: true,
        sort: 'asc',
        owner: mockOwner,
      });
  
      expect(result).toEqual([
        {
          id: 'mocktxid1i0',
          number: 1,
          owner: mockOwner,
          mediaContent: 'bW9ja0JvZHkx',
          mediaSize: 9,
          mediaType: 'text/plain',
          timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
          genesis: 'mocktxid1',
          outpoint: 'mocktxid1:0',
          fee: 0,
          height: 0,
          sat: 0,
          meta: mockMetadata
        }
      ]);
    });

    it('should throw an error when using unsupported filter conditions', async () => {
      await expect(instance.getInscriptions({
        creator: 'mockCreator', // Unsupported filter
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      })).rejects.toThrow('Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch');
  
      await expect(instance.getInscriptions({
        mimeType: 'image', // Unsupported filter
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      })).rejects.toThrow('Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch');
  
      await expect(instance.getInscriptions({
        mimeSubType: 'jpeg', // Unsupported filter
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      })).rejects.toThrow('Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch');
  
      await expect(instance.getInscriptions({
        outpoint: 'mocktxid:0', // Unsupported filter
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      })).rejects.toThrow('Unsupported filter types: creator, mimeType, mimeSubType, and outpoint are not supported by Rooch');
    });

    it('should handle empty result set', async () => {
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscriptions({
        owner: 'nonExistentOwner',
        limit: 10,
        decodeMetadata: false,
        sort: 'asc'
      });
  
      expect(result).toEqual([]);
    });

    it('should handle metadata decoding failure', async () => {
      const mockOwner = 'mockOwner1';
      const invalidEncodedMetadata = 'invalidBase64String';
  
      const mockInscriptions = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid1',
            index: 0,
            inscription_number: 1,
            body: '6d6f636b426f647931', // hex encoded 'mockBody1'
            content_type: 'text/plain',
            txid: 'mocktxid1',
            offset: '0',
            metadata: invalidEncodedMetadata,
            parents: '',
            is_curse: false,
            sequence_number: 0,
          },
          created_at: '2023-01-01T00:00:00Z',
          flag: 0,
          id: 'mock_id1',
          object_type: 'mock_type',
          size: '1',
          state_index: '0',
          tx_order: '0',
          updated_at: '2023-01-01T00:00:00Z'
        }
      ];
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: mockInscriptions,
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscriptions({
        limit: 10,
        decodeMetadata: true,
        sort: 'asc',
        owner: mockOwner
      });
  
      expect(result).toEqual([
        {
          id: 'mocktxid1i0',
          number: 1,
          owner: 'mockOwner1',
          mediaContent: 'bW9ja0JvZHkx',
          mediaSize: 9,
          mediaType: 'text/plain',
          timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
          genesis: 'mocktxid1',
          outpoint: 'mocktxid1:0',
          fee: 0,
          height: 0,
          sat: 0,
          // Note: 'meta' field should not be present due to decoding failure
        }
      ]);
    });
  });

 
  describe('getTransaction', () => {

    it('should successfully get transaction information including block data', async () => {
      const mockTxId = '0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131';
      const mockTxResponse = {
        vm_status: "Executed",
        return_values: [
          {
            value: {
              type_tag: "0x1::option::Option<0x4::types::Transaction>",
              value: "0x0150d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131020000006600000001e9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c870000000000fdffffff0247304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801210248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6025b1010240100000022512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710122020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710100e1f50500000000225120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07220201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
            },
            decoded_value: {
              abilities: 7,
              type: "0x1::option::Option<0x4::types::Transaction>",
              value: {
                vec: [
                  {
                    abilities: 7,
                    type: "0x4::types::Transaction",
                    value: {
                      id: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
                      input: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxIn",
                          value: {
                            previous_output: {
                              abilities: 7,
                              type: "0x4::types::OutPoint",
                              value: {
                                txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
                                vout: 0
                              }
                            },
                            script_sig: "0x",
                            sequence: 4294967293,
                            witness: {
                              abilities: 7,
                              type: "0x4::types::Witness",
                              value: {
                                witness: [
                                  "0x304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
                                  "0x0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
                                ]
                              }
                            }
                          }
                        }
                      ],
                      lock_time: 102,
                      output: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            value: "4899999835"
                          }
                        },
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x0201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            value: "100000000"
                          }
                        }
                      ],
                      version: 2
                    }
                  }
                ]
              }
            }
          }
        ]
      };
  
      // Mock response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      
      // Mock response for getting transaction height
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        result: {
          return_values: [
            {
              decoded_value: 12345
            }
          ]
        }
      });
  
      // Mock response for getting block information
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        result: {
          return_values: [
            {
              decoded_value: {
                prev_blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
                time: 1623456789
              }
            }
          ]
        }
      });
  
      const result = await instance.getTransaction({ txId: mockTxId });
  
      expect(result.tx).toEqual({
        txid: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        hash: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        version: 2,
        size: 0,
        vsize: 0,
        weight: 0,
        locktime: 102,
        vin: [
          {
            txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
            vout: 0,
            scriptSig: {
              asm: '',
              hex: ''
            },
            txinwitness: [
              "304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
              "0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
            ],
            sequence: 4294967293,
            value: 0
          }
        ],
        vout: [
          {
            value: 4899999835,
            n: 0,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 4899999835,
            scriptPubKey: {
              address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
              asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
              desc: "Script witness_v1_taproot",
              hex: '512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101',
              type: 'witness_v1_taproot',
            }
          },
          {
            value: 100000000,
            n: 1,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 100000000,
            scriptPubKey: {
              address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
              asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              desc: "Script witness_v1_taproot",
              hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              type: "witness_v1_taproot"
            }
          }
        ],
        blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
        blockheight: 12345,
        blocktime: 1623456789,
        confirmations: 0,
        time: 0,
        fee: 0
      });
    });

    it('should throw an error when transaction is not found', async () => {
      const mockTxId = '0x1234567890123456789012345678901234567890123456789012345678901234';
    
      // Mock response for getting transaction (empty result)
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        jsonrpc: "2.0",
        result: {
          vm_status: "Executed",
          return_values: [
            {
              value: {
                type_tag: "0x1::option::Option<0x4::types::Transaction>",
                value: "0x00"  // Empty option
              },
              decoded_value: {
                abilities: 7,
                type: "0x1::option::Option<0x4::types::Transaction>",
                value: {
                  vec: []
                }
              }
            }
          ]
        },
        id: 3908
      });
    
      await expect(instance.getTransaction({ txId: mockTxId })).rejects.toThrow(`Transaction with id ${mockTxId} not found`);
    });

    it('should return transaction without block info when unable to fetch block data', async () => {
      const mockTxId = '0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131';
      const mockTxResponse = {
        vm_status: "Executed",
        return_values: [
          {
            value: {
              type_tag: "0x1::option::Option<0x4::types::Transaction>",
              value: "0x0150d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131020000006600000001e9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c870000000000fdffffff0247304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801210248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6025b1010240100000022512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710122020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710100e1f50500000000225120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07220201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
            },
            decoded_value: {
              abilities: 7,
              type: "0x1::option::Option<0x4::types::Transaction>",
              value: {
                vec: [
                  {
                    abilities: 7,
                    type: "0x4::types::Transaction",
                    value: {
                      id: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
                      input: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxIn",
                          value: {
                            previous_output: {
                              abilities: 7,
                              type: "0x4::types::OutPoint",
                              value: {
                                txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
                                vout: 0
                              }
                            },
                            script_sig: "0x",
                            sequence: 4294967293,
                            witness: {
                              abilities: 7,
                              type: "0x4::types::Witness",
                              value: {
                                witness: [
                                  "0x304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
                                  "0x0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
                                ]
                              }
                            }
                          }
                        }
                      ],
                      lock_time: 102,
                      output: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            value: "4899999835"
                          }
                        },
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x0201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            value: "100000000"
                          }
                        }
                      ],
                      version: 2
                    }
                  }
                ]
              }
            }
          }
        ]
      };
    
      // Mock response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      
      // Mock response for getting transaction height (empty result)
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        result: {
          return_values: []
        }
      });
    
      const result = await instance.getTransaction({ txId: mockTxId });
    
      expect(result.tx).toEqual({
        txid: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        hash: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        version: 2,
        size: 0,
        vsize: 0,
        weight: 0,
        locktime: 102,
        vin: [
          {
            txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
            vout: 0,
            scriptSig: {
              asm: '',
              hex: ''
            },
            txinwitness: [
              "304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
              "0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
            ],
            sequence: 4294967293,
            value: 0
          }
        ],
        vout: [
          {
            value: 4899999835,
            n: 0,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 4899999835,
            scriptPubKey: {
              address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
              asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
              desc: "Script witness_v1_taproot",
              hex: '512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101',
              type: 'witness_v1_taproot',
            }
          },
          {
            value: 100000000,
            n: 1,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 100000000,
            scriptPubKey: {
              address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
              asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              desc: "Script witness_v1_taproot",
              hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              type: "witness_v1_taproot"
            }
          }
        ],
        blockhash: '',
        blockheight: 0,
        blocktime: 0,
        confirmations: 0,
        time: 0,
        fee: 0
      });
    });

    it('should return transaction with hex when hex parameter is true', async () => {
      const mockTxId = '0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131';
      const mockTxResponse = {
        vm_status: "Executed",
        return_values: [
          {
            value: {
              type_tag: "0x1::option::Option<0x4::types::Transaction>",
              value: "0x0150d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131020000006600000001e9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c870000000000fdffffff0247304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801210248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6025b1010240100000022512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710122020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710100e1f50500000000225120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07220201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
            },
            decoded_value: {
              abilities: 7,
              type: "0x1::option::Option<0x4::types::Transaction>",
              value: {
                vec: [
                  {
                    abilities: 7,
                    type: "0x4::types::Transaction",
                    value: {
                      id: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
                      input: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxIn",
                          value: {
                            previous_output: {
                              abilities: 7,
                              type: "0x4::types::OutPoint",
                              value: {
                                txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
                                vout: 0
                              }
                            },
                            script_sig: "0x",
                            sequence: 4294967293,
                            witness: {
                              abilities: 7,
                              type: "0x4::types::Witness",
                              value: {
                                witness: [
                                  "0x304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
                                  "0x0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
                                ]
                              }
                            }
                          }
                        }
                      ],
                      lock_time: 102,
                      output: [
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                              }
                            },
                            value: "4899999835"
                          }
                        },
                        {
                          abilities: 7,
                          type: "0x4::types::TxOut",
                          value: {
                            recipient_address: {
                              abilities: 7,
                              type: "0x3::bitcoin_address::BitcoinAddress",
                              value: {
                                bytes: "0x0201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            script_pubkey: {
                              abilities: 7,
                              type: "0x4::script_buf::ScriptBuf",
                              value: {
                                bytes: "0x5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                              }
                            },
                            value: "100000000"
                          }
                        }
                      ],
                      version: 2
                    }
                  }
                ]
              }
            }
          }
        ]
      };
    
      // Mock response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      
      // Mock response for getting transaction height
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        result: {
          return_values: [
            {
              decoded_value: 12345
            }
          ]
        }
      });
    
      // Mock response for getting block information
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        result: {
          return_values: [
            {
              decoded_value: {
                prev_blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
                time: 1623456789
              }
            }
          ]
        }
      });
    
      const result = await instance.getTransaction({ txId: mockTxId, hex: true });
    
      expect(result.tx).toMatchObject({
        txid: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        hash: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
        version: 2,
        size: 0,
        vsize: 0,
        weight: 0,
        locktime: 102,
        vin: [
          {
            txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
            vout: 0,
            scriptSig: {
              asm: '',
              hex: ''
            },
            txinwitness: [
              "304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
              "0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
            ],
            sequence: 4294967293,
            value: 0
          }
        ],
        vout: [
          {
            value: 4899999835,
            n: 0,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 4899999835,
            scriptPubKey: {
              address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
              asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
              desc: "Script witness_v1_taproot",
              hex: '512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101',
              type: 'witness_v1_taproot',
            }
          },
          {
            value: 100000000,
            n: 1,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 100000000,
            scriptPubKey: {
              address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
              asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              desc: "Script witness_v1_taproot",
              hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
              type: "witness_v1_taproot"
            }
          }
        ],
        blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
        blockheight: 12345,
        blocktime: 1623456789,
        confirmations: 0,
        time: 0,
        fee: 0,
        hex: expect.any(String)  // We expect the hex field to be present and be a string
      });
    
      // Verify that the hex field is present and is a non-empty string
      expect(result.tx.hex).toBeDefined();
      expect(typeof result.tx.hex).toBe('string');
    });

    it('should handle unexpected errors gracefully', async () => {
      const mockTxId = '0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131';
    
      // Mock a network error
      mockTransport.setMockResponse('rooch_executeViewFunction', () => {
        throw new Error('Network error');
      });
    
      await expect(instance.getTransaction({ txId: mockTxId })).rejects.toThrow('Transaction with id 0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131 not found');
    
      // Mock a server error
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        jsonrpc: "2.0",
        error: {
          code: -32000,
          message: "Server error"
        },
        id: 3907
      });
    
      await expect(instance.getTransaction({ txId: mockTxId })).rejects.toThrow('Transaction with id 0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131 not found');
    
      // Mock an unexpected response format
      mockTransport.setMockResponse('rooch_executeViewFunction', {
        jsonrpc: "2.0",
        result: {
          vm_status: "Executed",
          return_values: [
            {
              value: {
                type_tag: "0x1::option::Option<0x4::types::Transaction>",
                value: "Invalid data"
              },
              decoded_value: null
            }
          ]
        },
        id: 3907
      });
    
      await expect(instance.getTransaction({ txId: mockTxId })).rejects.toThrow('Transaction with id 0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131 not found');
    });
  });

 
  describe('getSpendables', () => {
    const mockTxResponse = {
      vm_status: "Executed",
      return_values: [
        {
          value: {
            type_tag: "0x1::option::Option<0x4::types::Transaction>",
            value: "0x0150d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131020000006600000001e9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c870000000000fdffffff0247304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801210248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6025b1010240100000022512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710122020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710100e1f50500000000225120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07220201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
          },
          decoded_value: {
            abilities: 7,
            type: "0x1::option::Option<0x4::types::Transaction>",
            value: {
              vec: [
                {
                  abilities: 7,
                  type: "0x4::types::Transaction",
                  value: {
                    id: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
                    input: [
                      {
                        abilities: 7,
                        type: "0x4::types::TxIn",
                        value: {
                          previous_output: {
                            abilities: 7,
                            type: "0x4::types::OutPoint",
                            value: {
                              txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
                              vout: 0
                            }
                          },
                          script_sig: "0x",
                          sequence: 4294967293,
                          witness: {
                            abilities: 7,
                            type: "0x4::types::Witness",
                            value: {
                              witness: [
                                "0x304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
                                "0x0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
                              ]
                            }
                          }
                        }
                      }
                    ],
                    lock_time: 102,
                    output: [
                      {
                        abilities: 7,
                        type: "0x4::types::TxOut",
                        value: {
                          recipient_address: {
                            abilities: 7,
                            type: "0x3::bitcoin_address::BitcoinAddress",
                            value: {
                              bytes: "0x020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                            }
                          },
                          script_pubkey: {
                            abilities: 7,
                            type: "0x4::script_buf::ScriptBuf",
                            value: {
                              bytes: "0x512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                            }
                          },
                          value: "4899999835"
                        }
                      },
                      {
                        abilities: 7,
                        type: "0x4::types::TxOut",
                        value: {
                          recipient_address: {
                            abilities: 7,
                            type: "0x3::bitcoin_address::BitcoinAddress",
                            value: {
                              bytes: "0x0201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                            }
                          },
                          script_pubkey: {
                            abilities: 7,
                            type: "0x4::script_buf::ScriptBuf",
                            value: {
                              bytes: "0x5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                            }
                          },
                          value: "100000000"
                        }
                      }
                    ],
                    version: 2
                  }
                }
              ]
            }
          }
        }
      ]
    };

    const mockTxId1 = '316168cb125dc8f2521bdd3f064ee41ca850a1a7cf8ddf294d45ac81b168d851';
    const mockTxId2 = '316168cb125dc8f2521bdd3f064ee41ca850a1a7cf8ddf294d45ac81b168d852';
    const mockTxId3 = '316168cb125dc8f2521bdd3f064ee41ca850a1a7cf8ddf294d45ac81b168d852';

    beforeEach(async()=>{
      // Mock response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
    });

    it('should successfully get spendable UTXOs', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const result = await instance.getSpendables({ address: mockAddress, value: 1500000 });

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        txid: mockTxId1,
        n: 0,
        sats: 1000000,
        scriptPubKey: {
          address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
          asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          desc: "Script witness_v1_taproot",
          hex: "512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          type: "witness_v1_taproot",
        },
        seals: {}
      });
      expect(result[1]).toEqual({
        txid: mockTxId2,
        n: 1,
        sats: 2000000,
        scriptPubKey: {
          address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
          asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          desc: "Script witness_v1_taproot",
          hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          type: "witness_v1_taproot",
        },
        seals: {}
      });
    });

    it('should return an empty array when no spendable UTXOs are available', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const result = await instance.getSpendables({ address: mockAddress, value: 1000000 });

      expect(result).toHaveLength(0);
    });

    it('should return all available UTXOs when requested value exceeds total available', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const result = await instance.getSpendables({ address: mockAddress, value: 5000000 });

      expect(result).toHaveLength(2);
      expect(result[0].sats).toBe(1000000);
      expect(result[1].sats).toBe(2000000);
    });

    it('should correctly handle pagination when fetching UTXOs', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOsPage1: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: true,
        next_cursor: { state_index: '1', tx_order: '1' }
      };

      const mockUTXOsPage2: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage1);
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage2);

      const result = await instance.getSpendables({ address: mockAddress, value: 3000000 });

      expect(result).toHaveLength(2);
      expect(result[0].txid).toBe(mockTxId1);
      expect(result[0].sats).toBe(1000000);
      expect(result[1].txid).toBe(mockTxId2);
      expect(result[1].sats).toBe(2000000);
    });

    it('should correctly limit the number of returned UTXOs based on the limit parameter', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId3,
              seals: {},
              txid: mockTxId3,
              value: '3000000',
              vout: 2
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id3',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '2',
            tx_order: '2',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const result = await instance.getSpendables({ address: mockAddress, value: 10000000, limit: 2 });

      expect(result).toHaveLength(2);
      expect(result[0].txid).toBe(mockTxId1);
      expect(result[0].sats).toBe(1000000);
      expect(result[1].txid).toBe(mockTxId2);
      expect(result[1].sats).toBe(2000000);
    });

    it('should throw an error when rarity parameter is provided', async () => {
      const mockAddress = 'mockAddress';
      
      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: 1000000, 
        rarity: ['common']  // Providing a rarity parameter
      })).rejects.toThrow('Rarity options are not supported for Rooch getSpendables');
    });

    it('should throw an error when filter parameter is provided', async () => {
      const mockAddress = 'mockAddress';
      
      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: 1000000, 
        filter: ['some_filter']  // Providing a filter parameter
      })).rejects.toThrow('filter options are not supported for Rooch getSpendables');
    });

    it('should throw an error when invalid address is provided', async () => {
      await expect(instance.getSpendables({ 
        address: '', 
        value: 1000000
      })).rejects.toThrow('Invalid address provided');

      await expect(instance.getSpendables({ 
        address: 123 as any, 
        value: 1000000
      })).rejects.toThrow('Invalid address provided');
    });

    it('should throw an error when invalid value is provided', async () => {
      const mockAddress = 'mockAddress';

      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: -1000
      })).rejects.toThrow('Invalid value provided');

      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: 'not a number' as any
      })).rejects.toThrow('Invalid value provided');
    });

    it('should correctly handle "all" and "spendable" type parameter', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {'some_seal_data':[]},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const resultAll = await instance.getSpendables({ 
        address: mockAddress, 
        value: 3000000, 
        type: 'all'
      });

      expect(resultAll).toHaveLength(2);

      const resultSpendable = await instance.getSpendables({ 
        address: mockAddress, 
        value: 3000000, 
        type: 'spendable'
      });

      expect(resultSpendable).toHaveLength(1);
      expect(resultSpendable[0].txid).toBe(mockTxId1);
    });
    
  });


  describe('getUnspents', () => {
    const mockTxResponse = {
      vm_status: "Executed",
      return_values: [
        {
          value: {
            type_tag: "0x1::option::Option<0x4::types::Transaction>",
            value: "0x0150d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131020000006600000001e9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c870000000000fdffffff0247304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801210248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6025b1010240100000022512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710122020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea5710100e1f50500000000225120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07220201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
          },
          decoded_value: {
            abilities: 7,
            type: "0x1::option::Option<0x4::types::Transaction>",
            value: {
              vec: [
                {
                  abilities: 7,
                  type: "0x4::types::Transaction",
                  value: {
                    id: "0x50d868b181ac454d29df8dcfa7a150a81ce44e063fdd1b52f2c85d12cb686131",
                    input: [
                      {
                        abilities: 7,
                        type: "0x4::types::TxIn",
                        value: {
                          previous_output: {
                            abilities: 7,
                            type: "0x4::types::OutPoint",
                            value: {
                              txid: "0xe9144d43dd0f52bfcc7044f61ae04d7592537326110775db23df7926afc83c87",
                              vout: 0
                            }
                          },
                          script_sig: "0x",
                          sequence: 4294967293,
                          witness: {
                            abilities: 7,
                            type: "0x4::types::Witness",
                            value: {
                              witness: [
                                "0x304402200b66a648b0dac1b5871758399d574200adba581794493de6465b4333477cd02a022012b5e02612646937e86a9625707fc90517d2a407a98bff826566320d729f378801",
                                "0x0248504d1c93def5d474becee5e2cbf515d9e9b884ed515b8ae2c4b41d8b7a7ad6"
                              ]
                            }
                          }
                        }
                      }
                    ],
                    lock_time: 102,
                    output: [
                      {
                        abilities: 7,
                        type: "0x4::types::TxOut",
                        value: {
                          recipient_address: {
                            abilities: 7,
                            type: "0x3::bitcoin_address::BitcoinAddress",
                            value: {
                              bytes: "0x020189f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                            }
                          },
                          script_pubkey: {
                            abilities: 7,
                            type: "0x4::script_buf::ScriptBuf",
                            value: {
                              bytes: "0x512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101"
                            }
                          },
                          value: "4899999835"
                        }
                      },
                      {
                        abilities: 7,
                        type: "0x4::types::TxOut",
                        value: {
                          recipient_address: {
                            abilities: 7,
                            type: "0x3::bitcoin_address::BitcoinAddress",
                            value: {
                              bytes: "0x0201b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                            }
                          },
                          script_pubkey: {
                            abilities: 7,
                            type: "0x4::script_buf::ScriptBuf",
                            value: {
                              bytes: "0x5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07"
                            }
                          },
                          value: "100000000"
                        }
                      }
                    ],
                    version: 2
                  }
                }
              ]
            }
          }
        }
      ]
    };

    const mockTxId1 = '316168cb125dc8f2521bdd3f064ee41ca850a1a7cf8ddf294d45ac81b168d851';
    const mockTxId2 = '316168cb125dc8f2521bdd3f064ee41ca850a1a7cf8ddf294d45ac81b168d852';

    beforeEach(async()=>{
      // Mock response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
    });

    it('should successfully get unspent UTXOs', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {'some_seal_data':[]},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      const result = await instance.getUnspents({ address: mockAddress });

      expect(result.totalUTXOs).toBe(2);
      expect(result.spendableUTXOs).toHaveLength(1);
      expect(result.unspendableUTXOs).toHaveLength(1);

      expect(result.spendableUTXOs[0]).toEqual({
        n: 0,
        txid: mockTxId1,
        sats: 1000000,
        scriptPubKey: {
          address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
          asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          desc: "Script witness_v1_taproot",
          hex: "512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          type: "witness_v1_taproot",
        },
        safeToSpend: true,
        confirmation: -1,
        seals: {}
      });

      expect(result.unspendableUTXOs[0]).toEqual({
        n: 1,
        txid: mockTxId2,
        sats: 2000000,
        scriptPubKey: {
          address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
          asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          desc: "Script witness_v1_taproot",
          hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          type: "witness_v1_taproot",
        },
        safeToSpend: false,
        confirmation: -1,
        seals: {"some_seal_data":[]}
      });
    });

    it('should handle pagination correctly', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOsPage1: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: true,
        next_cursor: { state_index: '1', tx_order: '1' }
      };
    
      const mockUTXOsPage2: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {'some_seal_data':[]},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage1);
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage2);
    
      const result = await instance.getUnspents({ address: mockAddress, limit: 2 });
    
      expect(result.totalUTXOs).toBe(2);
      expect(result.spendableUTXOs).toHaveLength(1);
      expect(result.unspendableUTXOs).toHaveLength(1);
    
      expect(result.spendableUTXOs[0]).toEqual({
        n: 0,
        txid: mockTxId1,
        sats: 1000000,
        scriptPubKey: {
          address: "bcrt1p38ma55ap67wu4hlsf2ey5p0dvzmq2wzc7zda3dp3mm39kr49wyqs96d69e",
          asm: "OP_1 89f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          desc: "Script witness_v1_taproot",
          hex: "512089f7da53a1d79dcadff04ab24a05ed60b6053858f09bd8b431dee25b0ea57101",
          type: "witness_v1_taproot",
        },
        safeToSpend: true,
        confirmation: -1,
        seals: {}
      });
    
      expect(result.unspendableUTXOs[0]).toEqual({
        n: 1,
        txid: mockTxId2,
        sats: 2000000,
        scriptPubKey: {
          address: "bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k",
          asm: "OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          desc: "Script witness_v1_taproot",
          hex: "5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07",
          type: "witness_v1_taproot",
        },
        safeToSpend: false,
        confirmation: -1,
        seals: {"some_seal_data":[]}
      });
    });

    it('should handle empty result set', async () => {
      const mockAddress = 'mockAddress';
      const mockEmptyUTXOs: PaginatedUTXOStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockEmptyUTXOs);
    
      const result = await instance.getUnspents({ address: mockAddress });
    
      expect(result.totalUTXOs).toBe(0);
      expect(result.spendableUTXOs).toHaveLength(0);
      expect(result.unspendableUTXOs).toHaveLength(0);
    });

    it('should correctly handle "all" and "spendable" type parameter', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {'some_seal_data':[]},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);
    
      const resultAll = await instance.getUnspents({ 
        address: mockAddress, 
        type: 'all'
      });
    
      expect(resultAll.totalUTXOs).toBe(2);
      expect(resultAll.spendableUTXOs).toHaveLength(1);
      expect(resultAll.unspendableUTXOs).toHaveLength(1);
    
      const resultSpendable = await instance.getUnspents({ 
        address: mockAddress, 
        type: 'spendable'
      });
    
      expect(resultSpendable.totalUTXOs).toBe(2);
      expect(resultSpendable.spendableUTXOs).toHaveLength(1);
      expect(resultSpendable.unspendableUTXOs).toHaveLength(0);
      expect(resultSpendable.spendableUTXOs[0].txid).toBe(mockTxId1);
    });

    it('should correctly handle ascending and descending sorting', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);
    
      const resultAsc = await instance.getUnspents({ 
        address: mockAddress, 
        sort: 'asc'
      });
    
      expect(resultAsc.spendableUTXOs[0].sats).toBe(1000000);
      expect(resultAsc.spendableUTXOs[1].sats).toBe(2000000);
    
      const resultDesc = await instance.getUnspents({ 
        address: mockAddress, 
        sort: 'desc'
      });
    
      expect(resultDesc.spendableUTXOs[0].sats).toBe(2000000);
      expect(resultDesc.spendableUTXOs[1].sats).toBe(1000000);
    });
    
    it('should correctly apply limit parameter', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);
    
      const result = await instance.getUnspents({ 
        address: mockAddress, 
        limit: 1
      });
    
      expect(result.totalUTXOs).toBe(1);
      expect(result.spendableUTXOs).toHaveLength(1);
      expect(result.spendableUTXOs[0].sats).toBe(1000000);
    });
    
    it('should correctly use next parameter for pagination', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOsPage1: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: '1000000',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: true,
        next_cursor: { state_index: '1', tx_order: '1' }
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage1);
    
      const result1 = await instance.getUnspents({ 
        address: mockAddress, 
        limit: 1
      });
    
      expect(result1.totalUTXOs).toBe(1);
      expect(result1.spendableUTXOs).toHaveLength(1);
      expect(result1.spendableUTXOs[0].sats).toBe(1000000);
    
      mockTransport.resetMocks();

      const mockUTXOsPage2: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: '2000000',
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };
    
      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOsPage2);
      mockTransport.setMockResponse('rooch_executeViewFunction', mockTxResponse);
    
      const result2 = await instance.getUnspents({ 
        address: mockAddress, 
        limit: 1,
        next: JSON.stringify(mockUTXOsPage1.next_cursor)
      });
    
      expect(result2.totalUTXOs).toBe(1);
      expect(result2.spendableUTXOs).toHaveLength(1);
      expect(result2.spendableUTXOs[0].sats).toBe(2000000);
    });
    
    it('should throw an error for invalid address', async () => {
      await expect(instance.getUnspents({ address: '' }))
        .rejects.toThrow('Invalid address provided');
    
      await expect(instance.getUnspents({ address: 123 as any }))
        .rejects.toThrow('Invalid address provided');
    });
    
    it('should throw an error for unsupported rarity parameter', async () => {
      await expect(instance.getUnspents({ 
        address: 'mockAddress', 
        rarity: ['common'] as any
      })).rejects.toThrow('Rarity options are not supported for Rooch getUnspents');
    });
    
    it('should handle large UTXO values and throw error for values exceeding safe integer range', async () => {
      const mockAddress = 'mockAddress';
      const largeValue = '9007199254740992'; // Number.MAX_SAFE_INTEGER + 1
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: mockTxId1,
              seals: {},
              txid: mockTxId1,
              value: largeValue,
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id1',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          },
          {
            value: {
              bitcoin_txid: mockTxId2,
              seals: {},
              txid: mockTxId2,
              value: largeValue,
              vout: 1
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id2',
            object_type: 'mock_type',
            owner: mockAddress,
            owner_bitcoin_address: mockAddress,
            size: '1',
            state_index: '1',
            tx_order: '1',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockUTXOs);

      await expect(instance.getUnspents({ address: mockAddress }))
        .rejects.toThrow('Invalid UTXO value: 9007199254740992');
    });
  });

  describe('relay', () => {
    it('should successfully send a transaction and return the transaction hash', async () => {
      const mockTxHex = '0123456789abcdef';
      const mockTxHash = '9876543210fedcba';

      mockTransport.setMockResponse('btc_broadcastTX', mockTxHash);

      const result = await instance.relay({ hex: mockTxHex });

      expect(result).toBe(mockTxHash);
    });

    it('should throw an error for invalid transaction hex', async () => {
      const invalidTxHex = 'invalid_hex';
  
      await expect(instance.relay({ hex: invalidTxHex }))
        .rejects.toThrow('Invalid transaction hex');
    });

    it('should successfully send a transaction with maxFeeRate', async () => {
      const mockTxHex = '0123456789abcdef';
      const mockTxHash = 'fedcba9876543210';
      const mockMaxFeeRate = 10;
  
      mockTransport.setMockResponse('btc_broadcastTX', mockTxHash);
  
      const result = await instance.relay({ hex: mockTxHex, maxFeeRate: mockMaxFeeRate });
  
      expect(result).toBe(mockTxHash);
    });

    it('should throw an error when broadcastBitcoinTX fails', async () => {
      const mockTxHex = '0123456789abcdef';
      const mockError = new Error('Network error');
  
      mockTransport.setMockErrorResponse('btc_broadcastTX', mockError);
  
      await expect(instance.relay({ hex: mockTxHex }))
        .rejects.toThrow('Failed to broadcast transaction: Error: Network error');

    });
    
    it('should throw an error when validate option is provided', async () => {
      const mockTxHex = '0123456789abcdef';
  
      await expect(instance.relay({ hex: mockTxHex, validate: true }))
        .rejects.toThrow('validate options are not supported for Rooch broadcastBitcoinTX');
    });

  });

});
