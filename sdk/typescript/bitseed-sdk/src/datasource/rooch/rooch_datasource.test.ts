import cbor from 'cbor'
import { RoochDataSource } from './rooch_datasource';
import { Wallet } from '../../wallet';
import { 
  RoochTransport, 
  PaginatedUTXOStateViews, 
  PaginatedInscriptionStateViews,
  UTXOStateView,
} from '@roochnetwork/rooch-sdk';

class MockRoochTransport implements RoochTransport {
  private mockResponses: Map<string, any[]> = new Map();
  private callCounts: Map<string, number> = new Map();

  setMockResponse(method: string, response: any) {
    if (!this.mockResponses.has(method)) {
      this.mockResponses.set(method, []);
    }
    this.mockResponses.get(method)!.push(response);
    this.callCounts.set(method, 0);
  }

  async request<T>({ method }: { method: string; params: unknown[] }): Promise<T> {
    const responses = this.mockResponses.get(method);
    if (responses && responses.length > 0) {
      const callCount = this.callCounts.get(method) || 0;
      const response = responses[callCount % responses.length];
      this.callCounts.set(method, callCount + 1);
      return response as T;
    }
    throw new Error(`No mock response set for method: ${method}`);
  }

  resetMocks() {
    this.mockResponses.clear();
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
              seals: 'mock_seals',
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
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: 'bW9ja01ldGFkYXRh', // base64 encoded 'mockMetadata'
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
      const base64EncodedMetadata = Buffer.from(encodedMetadata).toString('base64');

      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
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

      jest.spyOn(console, 'warn').mockImplementation(() => {});
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
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
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
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
          seals: 'mock_seals',
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
          seals: 'mock_seals',
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
          seals: 'mock_seals',
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
            body: 'bW9ja0JvZHkx', // base64 encoded 'mockBody1'
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
            body: 'bW9ja0JvZHkx', // base64 encoded 'mockBody1'
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
            body: 'bW9ja0JvZHky', // base64 encoded 'mockBody2'
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
      const base64EncodedMetadata = Buffer.from(encodedMetadata).toString('base64');
  
      const mockInscriptions = [
        {
          owner: mockOwner,
          value: {
            bitcoin_txid: 'mocktxid1',
            index: 0,
            inscription_number: 1,
            body: 'bW9ja0JvZHkx', // base64 encoded 'mockBody1'
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
            body: 'bW9ja0JvZHkx', // base64 encoded 'mockBody1'
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
      const mockTxId = '0x1234567890123456789012345678901234567890123456789012345678901234';
      const mockTx = {
        id: mockTxId,
        version: 1,
        lock_time: 0,
        input: [
          {
            previous_output: { 
              txid: '0x2345678901234567890123456789012345678901234567890123456789012345', 
              vout: 0 
            },
            script_sig: Buffer.from('script_sig').toString('base64'),
            sequence: 4294967295,
            witness: { witness: ['witness_data'] }
          }
        ],
        output: [
          {
            value: '100000000',
            script_pubkey: Buffer.from('script_pubkey').toString('base64'),
            recipient_address: 'tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx'
          }
        ]
      };

      // Set mock responses for all three executeViewFunction calls
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: mockTx }] });
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: 12345 }] });
      mockTransport.setMockResponse('rooch_executeViewFunction', { 
        return_values: [{ 
          decoded_value: { 
            prev_blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
            time: 1623456789
          } 
        }] 
      });

      const result = await instance.getTransaction({ txId: mockTxId });

      expect(result.tx).toEqual({
        txid: mockTxId,
        hash: mockTxId,
        version: 1,
        size: 0,
        vsize: 0,
        weight: 0,
        locktime: 0,
        vin: [
          {
            txid: '0x2345678901234567890123456789012345678901234567890123456789012345',
            vout: 0,
            scriptSig: {
              asm: '',
              hex: '7363726970745f736967'
            },
            txinwitness: ['witness_data'],
            sequence: 4294967295,
            value: 0
          }
        ],
        vout: [
          {
            value: 100000000,
            n: 0,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 100000000,
            scriptPubKey: {
              asm: '',
              desc: '',
              hex: '7363726970745f7075626b6579',
              type: 'unknown',
              address: 'tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx'
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

      // Mock an empty response to simulate a not found transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [] });

      await expect(instance.getTransaction({ txId: mockTxId }))
        .rejects.toThrow(`Transaction with id ${mockTxId} not found`);
    });

    it('should return transaction without block information when getting transaction height fails', async () => {
      const mockTxId = '0x1234567890123456789012345678901234567890123456789012345678901234';
      const mockTx = {
        id: mockTxId,
        version: 1,
        lock_time: 0,
        input: [],
        output: []
      };

      // Mock successful response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: mockTx }] });
      
      // Mock empty response for getting transaction height to simulate failure
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [] });

      const result = await instance.getTransaction({ txId: mockTxId });

      expect(result.tx).toEqual(expect.objectContaining({
        txid: mockTxId,
        hash: mockTxId,
        version: 1,
        locktime: 0,
        blockhash: "",
        blockheight: 0,
        blocktime: 0
      }));
    });

    it('should return transaction with empty hex when hex parameter is true', async () => {
      const mockTxId = '0x1234567890123456789012345678901234567890123456789012345678901234';
      const mockTx = {
        id: mockTxId,
        version: 1,
        lock_time: 0,
        input: [],
        output: []
      };

      // Mock successful response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: mockTx }] });
      
      // Mock successful response for getting transaction height
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: 12345 }] });

      // Mock successful response for getting block information
      mockTransport.setMockResponse('rooch_executeViewFunction', { 
        return_values: [{ 
          decoded_value: { 
            prev_blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
            time: 1623456789
          } 
        }] 
      });

      const result = await instance.getTransaction({ txId: mockTxId, hex: true });

      expect(result.tx).toEqual(expect.objectContaining({
        txid: mockTxId,
        hash: mockTxId,
        version: 1,
        locktime: 0,
        blockheight: 12345,
        blockhash: '0x3456789012345678901234567890123456789012345678901234567890123456',
        blocktime: 1623456789,
        hex: ""
      }));
    });

    it('should correctly handle transactions with multiple inputs and outputs', async () => {
      const mockTxId = '0x1234567890123456789012345678901234567890123456789012345678901234';
      const mockTx = {
        id: mockTxId,
        version: 1,
        lock_time: 0,
        input: [
          {
            previous_output: { 
              txid: '0x2345678901234567890123456789012345678901234567890123456789012345', 
              vout: 0 
            },
            script_sig: Buffer.from('script_sig_1').toString('base64'),
            sequence: 4294967295,
            witness: { witness: ['witness_data_1'] }
          },
          {
            previous_output: { 
              txid: '0x3456789012345678901234567890123456789012345678901234567890123456', 
              vout: 1 
            },
            script_sig: Buffer.from('script_sig_2').toString('base64'),
            sequence: 4294967294,
            witness: { witness: ['witness_data_2a', 'witness_data_2b'] }
          }
        ],
        output: [
          {
            value: '50000000',
            script_pubkey: Buffer.from('script_pubkey_1').toString('base64'),
            recipient_address: 'tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx'
          },
          {
            value: '49000000',
            script_pubkey: Buffer.from('script_pubkey_2').toString('base64'),
            recipient_address: 'tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7'
          }
        ]
      };

      // Mock successful response for getting transaction
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: mockTx }] });
      
      // Mock successful response for getting transaction height
      mockTransport.setMockResponse('rooch_executeViewFunction', { return_values: [{ decoded_value: 12345 }] });

      // Mock successful response for getting block information
      mockTransport.setMockResponse('rooch_executeViewFunction', { 
        return_values: [{ 
          decoded_value: { 
            prev_blockhash: '0x4567890123456789012345678901234567890123456789012345678901234567',
            time: 1623456789
          } 
        }] 
      });

      const result = await instance.getTransaction({ txId: mockTxId });

      expect(result.tx).toEqual(expect.objectContaining({
        txid: mockTxId,
        hash: mockTxId,
        version: 1,
        locktime: 0,
        vin: [
          {
            txid: '0x2345678901234567890123456789012345678901234567890123456789012345',
            vout: 0,
            scriptSig: {
              asm: '',
              hex: '7363726970745f7369675f31'
            },
            txinwitness: ['witness_data_1'],
            sequence: 4294967295,
            value: 0
          },
          {
            txid: '0x3456789012345678901234567890123456789012345678901234567890123456',
            vout: 1,
            scriptSig: {
              asm: '',
              hex: '7363726970745f7369675f32'
            },
            txinwitness: ['witness_data_2a', 'witness_data_2b'],
            sequence: 4294967294,
            value: 0
          }
        ],
        vout: [
          {
            value: 50000000,
            n: 0,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 50000000,
            scriptPubKey: {
              asm: '',
              desc: '',
              hex: '7363726970745f7075626b65795f31',
              type: 'unknown',
              address: 'tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx'
            }
          },
          {
            value: 49000000,
            n: 1,
            ordinals: [],
            inscriptions: [],
            spent: false,
            sats: 49000000,
            scriptPubKey: {
              asm: '',
              desc: '',
              hex: '7363726970745f7075626b65795f32',
              type: 'unknown',
              address: 'tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7'
            }
          }
        ],
        blockheight: 12345,
        blockhash: '0x4567890123456789012345678901234567890123456789012345678901234567',
        blocktime: 1623456789,
        confirmations: 0,
        fee: 0,
        size: 0,
        time: 0
      }));
    });

  });

  describe('getSpendables', () => {
    it('should successfully get spendable UTXOs', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: 'txid1',
              seals: '',
              txid: 'txid1',
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
              bitcoin_txid: 'txid2',
              seals: '',
              txid: 'txid2',
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
        txid: 'txid1',
        n: 0,
        sats: 1000000,
        scriptPubKey: {
          asm: '',
          desc: '',
          hex: '',
          address: mockAddress,
          type: 'p2tr'
        },
        seals: ""
      });
      expect(result[1]).toEqual({
        txid: 'txid2',
        n: 1,
        sats: 2000000,
        scriptPubKey: {
          asm: '',
          desc: '',
          hex: '',
          address: mockAddress,
          type: 'p2tr'
        },
        seals: ""
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
              bitcoin_txid: 'txid1',
              seals: '',
              txid: 'txid1',
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
              bitcoin_txid: 'txid2',
              seals: '',
              txid: 'txid2',
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
              bitcoin_txid: 'txid1',
              seals: '',
              txid: 'txid1',
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
              bitcoin_txid: 'txid2',
              seals: '',
              txid: 'txid2',
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
      expect(result[0].txid).toBe('txid1');
      expect(result[0].sats).toBe(1000000);
      expect(result[1].txid).toBe('txid2');
      expect(result[1].sats).toBe(2000000);
    });

    it('should correctly limit the number of returned UTXOs based on the limit parameter', async () => {
      const mockAddress = 'mockAddress';
      const mockUTXOs: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              bitcoin_txid: 'txid1',
              seals: '',
              txid: 'txid1',
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
              bitcoin_txid: 'txid2',
              seals: '',
              txid: 'txid2',
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
              bitcoin_txid: 'txid3',
              seals: '',
              txid: 'txid3',
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
      expect(result[0].txid).toBe('txid1');
      expect(result[0].sats).toBe(1000000);
      expect(result[1].txid).toBe('txid2');
      expect(result[1].sats).toBe(2000000);
    });

    it('should throw an error when rarity parameter is provided', async () => {
      const mockAddress = 'mockAddress';
      
      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: 1000000, 
        rarity: ['common']  // Providing a rarity parameter
      })).rejects.toThrow('Rarity and filter options are not supported for Rooch getSpendables');
    });

    it('should throw an error when filter parameter is provided', async () => {
      const mockAddress = 'mockAddress';
      
      await expect(instance.getSpendables({ 
        address: mockAddress, 
        value: 1000000, 
        filter: ['some_filter']  // Providing a filter parameter
      })).rejects.toThrow('Rarity and filter options are not supported for Rooch getSpendables');
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
              bitcoin_txid: 'txid1',
              seals: '',
              txid: 'txid1',
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
              bitcoin_txid: 'txid2',
              seals: 'some_seal_data',
              txid: 'txid2',
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
      expect(resultSpendable[0].txid).toBe('txid1');
    });
    
  });
});
